# Requirements Document

## Introduction

The Library Management system provides comprehensive media library organization, version tracking, metadata management, and maintenance capabilities for TorrentFlix. It is designed to efficiently manage large collections of movies and TV series on HDD-based storage with modest hardware, supporting multiple file versions per media item, intelligent deduplication, integrity checking, and safe cleanup operations.

## User Guide Context

This requirements document is informed by the detailed user guide at `docs/LIBRARY_MANAGEMENT_DOCS/USER_GUIDE_LIBRARY_MANAGEMENT.md`, which describes the primary user workflows:

1. **Library Setup**: Users add library roots and configure settings
2. **Import Workflow**: Users import files manually, via watch folders, or using import profiles
3. **Browsing and Discovery**: Users search, filter, and save searches with case-insensitive and fuzzy matching
4. **Media Item Management**: Users view, edit, and tag media items with custom metadata
5. **Version Management**: Users compare versions, set preferred versions, and identify duplicates
6. **Collections**: Users create manual and smart collections that auto-update
7. **Storage Management**: Users view analytics, identify duplicates, and execute cleanup
8. **Maintenance**: Users run rescans, integrity checks, view audit logs, and monitor jobs
9. **Playback Integration**: Users launch media players, track playback position, and manage watch status

All acceptance criteria in this document are designed to support these user workflows.

## Glossary

- **MediaItem**: A logical entity representing a unique movie or TV series (identified by title and year)
- **FileVersion**: A physical file on disk associated with a MediaItem, containing technical metadata and status information
- **Library Root**: A directory path designated as a storage location for media files
- **Staging Directory**: A temporary location where files are processed before being committed to the library
- **Fingerprint**: A hash value computed from file content for duplicate detection (fast-hash for quick checks, full-hash for exact matching)
- **Quality Label**: A classification of file quality based on resolution, codec, and source (e.g., "1080p BluRay x264")
- **Preferred Version**: A user-designated FileVersion that should be prioritized for playback or kept during cleanup
- **Soft Delete**: Marking a file as trashed without immediate physical deletion, allowing recovery
- **Rescan Job**: A background operation that reconciles filesystem state with database records
- **Integrity Check**: Verification that database records match actual filesystem state
- **Cleanup Candidate**: A FileVersion identified as potentially removable (duplicate, low-quality, or trashed)
- **Job Manager**: The background task scheduling and execution system
- **Audit Log**: A record of all file operations (moves, deletions, status changes) for traceability
- **Watch Folder**: A directory monitored for new files that are automatically imported
- **Symbolic Link**: A filesystem reference that points to another file or directory location
- **Hard Link**: A filesystem entry that shares the same inode as another file
- **Mount Point**: A directory where a filesystem or storage device is attached to the directory tree
- **Collection**: A user-defined grouping of MediaItems for organization
- **Smart Collection**: A dynamically updated Collection based on filter criteria
- **Export Profile**: A configuration for exporting library metadata and file lists
- **Import Profile**: A configuration for batch importing files with predefined settings

## Requirements

### Requirement 1: Media Item and File Version Management

**User Story:** As a user, I want to manage logical media items with multiple physical file versions, so that I can track different qualities and releases of the same content.

#### Acceptance Criteria

1. WHEN a new file is imported THEN the System SHALL create or update a MediaItem with normalized title and year metadata
2. WHEN a file is committed to the library THEN the System SHALL create a FileVersion record containing absolute path, file size, resolution, codec, container format, quality label, fingerprint, added timestamp, and status
3. WHEN multiple files represent the same media THEN the System SHALL associate all FileVersion records with the same MediaItem
4. WHEN a user views a MediaItem THEN the System SHALL display all associated FileVersion records with their technical metadata and status
5. WHERE a FileVersion is marked as preferred THEN the System SHALL store and display the preferred flag for that version

### Requirement 2: Import and Commit Pipeline

**User Story:** As a user, I want files to be safely imported through a staging process, so that incomplete or failed imports do not corrupt my library.

#### Acceptance Criteria

1. WHEN a file is selected for import THEN the System SHALL copy or move the file to the Staging Directory
2. WHILE a file is in staging THEN the System SHALL probe technical metadata including container format, resolution, codec, duration, and audio tracks
3. WHEN metadata probing completes THEN the System SHALL compute a fast fingerprint for the file
4. WHEN the user commits a staged file THEN the System SHALL atomically move the file to the Library Root using a consistent naming scheme
5. IF the atomic move fails THEN the System SHALL rollback the operation and retain the file in staging with error status
6. WHEN a file is successfully committed THEN the System SHALL insert or update database records for MediaItem and FileVersion with status "present"
7. WHEN a commit operation completes THEN the System SHALL remove the file from the Staging Directory

### Requirement 3: Background Hashing and Fingerprinting

**User Story:** As a user, I want exact duplicate detection through full-file hashing without impacting system performance, so that I can identify identical files across my library.

#### Acceptance Criteria

1. WHEN a FileVersion is created THEN the System SHALL mark the hashing status as "pending"
2. WHEN the Job Manager schedules a hashing job THEN the System SHALL compute a full-file hash for FileVersions with pending hashing status
3. WHILE hashing operations run THEN the System SHALL limit concurrency to one file at a time to avoid overloading disk I/O
4. WHEN a full-file hash is computed THEN the System SHALL store the hash value in the FileVersion record and update hashing status to "complete"
5. WHERE system resources are constrained THEN the System SHALL allow user configuration of hashing job scheduling and concurrency limits
6. WHEN multiple FileVersions have identical full-file hashes THEN the System SHALL identify them as exact duplicates

### Requirement 4: Library Browsing and Filtering

**User Story:** As a user, I want to browse and filter my media library efficiently, so that I can quickly find content without waiting for filesystem scans.

#### Acceptance Criteria

1. WHEN the user opens the library view THEN the System SHALL query the database and display MediaItems with title, year, poster thumbnail, version count, and total size
2. WHEN the user applies filters THEN the System SHALL query indexed database fields including title, year, resolution, quality label, status, user tags, and user rating
3. WHEN the user searches by text THEN the System SHALL perform a case-insensitive search on normalized title fields
4. WHEN the user expands a MediaItem THEN the System SHALL display all FileVersion records with path, resolution, codec, size, status, preferred flag, and timestamps
5. WHEN the user sorts the library THEN the System SHALL support sorting by title, year, added date, size, version count, and user rating
6. WHILE displaying the library THEN the System SHALL lazy-load poster images and detailed metadata only when visible or requested

### Requirement 5: Rescan and Integrity Checking

**User Story:** As a user, I want to reconcile filesystem changes with database records, so that manual file operations or external changes are detected and handled.

#### Acceptance Criteria

1. WHEN a rescan job is triggered THEN the System SHALL recursively walk all Library Root directories
2. WHEN a file exists on disk but not in the database THEN the System SHALL flag the file as a new import candidate
3. WHEN a FileVersion record references a path that no longer exists THEN the System SHALL update the status to "missing"
4. WHEN a MediaItem has all FileVersions marked as missing THEN the System SHALL flag the MediaItem as incomplete
5. WHILE a rescan job runs THEN the System SHALL process files in batches with progress tracking to avoid blocking operations
6. WHEN a rescan job completes THEN the System SHALL generate a summary report of new files, missing files, and status changes
7. WHERE the user requests an integrity check THEN the System SHALL verify that all FileVersion paths exist and file sizes match recorded values

### Requirement 6: Duplicate Detection and Version Management

**User Story:** As a user, I want to identify duplicate and variant versions of media, so that I can make informed decisions about which versions to keep.

#### Acceptance Criteria

1. WHEN the user requests duplicate detection THEN the System SHALL identify FileVersions with identical full-file hashes as exact duplicates
2. WHEN the user requests variant detection THEN the System SHALL identify FileVersions associated with the same MediaItem as variants
3. WHEN displaying duplicates or variants THEN the System SHALL show a comparison view with resolution, codec, size, quality label, and hash values
4. WHEN multiple versions exist for a MediaItem THEN the System SHALL calculate and display a quality score for each version based on resolution, codec, source, and file size
5. WHERE a user has not set a preferred version THEN the System SHALL recommend the highest-scoring version as the default
6. WHEN the user marks a version as preferred THEN the System SHALL override automatic quality scoring for that MediaItem

### Requirement 7: Soft Delete and Trash Management

**User Story:** As a user, I want deleted files to be recoverable for a grace period, so that accidental deletions can be reversed without data loss.

#### Acceptance Criteria

1. WHEN a user deletes a FileVersion THEN the System SHALL update the status to "trashed" and move the file to a designated Trash Directory
2. WHEN a file is moved to trash THEN the System SHALL record the deletion timestamp and original path in the FileVersion metadata
3. WHEN a file is moved to trash THEN the System SHALL create an Audit Log entry with the operation type, timestamp, affected file, and user or system identifier
4. WHILE a FileVersion is in trashed status THEN the System SHALL exclude it from library browsing by default
5. WHEN the user requests to view trashed items THEN the System SHALL display all FileVersions with trashed status and their deletion timestamps
6. WHERE a FileVersion has been trashed for less than the configured grace period THEN the System SHALL allow the user to restore the file to its original location
7. WHEN a FileVersion exceeds the trash grace period THEN the System SHALL mark it as a cleanup candidate for permanent deletion

### Requirement 8: Cleanup and Space Management

**User Story:** As a user, I want to identify and remove unnecessary files to free disk space, so that I can manage storage efficiently on HDD-based systems.

#### Acceptance Criteria

1. WHEN the user requests cleanup suggestions THEN the System SHALL identify cleanup candidates including exact duplicates, lower-quality variants, trashed files past grace period, and missing file records
2. WHEN displaying cleanup candidates THEN the System SHALL show the file path, size, reason for candidacy, and estimated space savings
3. WHEN the user selects cleanup candidates THEN the System SHALL calculate total space savings and display a confirmation summary
4. WHEN the user confirms cleanup THEN the System SHALL permanently delete selected files and remove corresponding FileVersion records
5. IF a cleanup operation fails for any file THEN the System SHALL log the error, skip that file, and continue with remaining files
6. WHEN a cleanup operation completes THEN the System SHALL generate a summary report of deleted files, freed space, and any errors
7. WHERE disk usage exceeds a configured threshold THEN the System SHALL notify the user and suggest running cleanup

### Requirement 9: User Metadata and Tagging

**User Story:** As a user, I want to add custom metadata and tags to media items, so that I can organize and filter my library according to personal preferences.

#### Acceptance Criteria

1. WHEN a user views a MediaItem THEN the System SHALL display editable fields for user rating, watch status, and custom notes
2. WHEN a user adds or modifies user metadata THEN the System SHALL persist the changes to the database immediately
3. WHEN a user adds tags to a MediaItem THEN the System SHALL store tags as a list of text values associated with the MediaItem
4. WHEN the user filters by tags THEN the System SHALL return all MediaItems containing any of the specified tags
5. WHEN the user searches by custom notes THEN the System SHALL perform a text search on the notes field
6. WHERE a user assigns a custom rating THEN the System SHALL display the user rating alongside external ratings from metadata providers

### Requirement 10: Storage Monitoring and Analytics

**User Story:** As a user, I want to view storage usage statistics and breakdowns, so that I can understand how disk space is allocated and identify optimization opportunities.

#### Acceptance Criteria

1. WHEN the user opens storage analytics THEN the System SHALL display total library size, number of MediaItems, number of FileVersions, and average versions per media
2. WHEN displaying storage breakdown THEN the System SHALL show space used by resolution category, quality label, and status (present, trashed, missing)
3. WHEN calculating duplicate space THEN the System SHALL identify space used by exact duplicates and lower-quality variants
4. WHEN displaying per-media storage THEN the System SHALL show the largest MediaItems by total size across all versions
5. WHEN the user requests a space savings estimate THEN the System SHALL calculate potential freed space from removing duplicates, trashed files, and low-quality variants
6. WHERE storage usage data is requested THEN the System SHALL compute statistics from database records without requiring filesystem scans

### Requirement 11: Background Job Management

**User Story:** As a user, I want to monitor and control background maintenance operations, so that I can schedule intensive tasks during idle periods and track their progress.

#### Acceptance Criteria

1. WHEN the user opens the jobs dashboard THEN the System SHALL display all scheduled, running, and completed jobs with type, status, progress percentage, and timestamps
2. WHEN a background job is running THEN the System SHALL update progress information at regular intervals
3. WHEN the user requests to start a job THEN the System SHALL validate prerequisites and enqueue the job for execution
4. WHEN the user requests to cancel a running job THEN the System SHALL signal the job to stop gracefully and update its status to cancelled
5. WHERE a job fails THEN the System SHALL log the error details and update the job status to failed with error message
6. WHEN a job completes successfully THEN the System SHALL update the job status to completed and store any result summary
7. WHILE intensive jobs run THEN the System SHALL throttle disk I/O operations to avoid impacting user operations like playback or browsing

### Requirement 12: Multi-Root Library Support

**User Story:** As a user, I want to designate multiple directories as library roots across different drives, so that I can organize media across multiple HDDs or partitions.

#### Acceptance Criteria

1. WHEN the user adds a Library Root THEN the System SHALL store the absolute path and assign a unique root identifier
2. WHEN storing FileVersion paths THEN the System SHALL record both the root identifier and relative path within that root
3. WHEN a Library Root is moved or remounted THEN the System SHALL allow the user to update the root path without losing FileVersion associations
4. WHEN scanning for files THEN the System SHALL recursively scan all configured Library Roots
5. WHERE a Library Root is designated as archive or cold storage THEN the System SHALL mark associated FileVersions with lower priority for background operations
6. WHEN displaying FileVersion information THEN the System SHALL resolve the full path by combining root path and relative path

### Requirement 13: Metadata Enrichment and Caching

**User Story:** As a user, I want media items enriched with descriptive metadata and artwork from external providers, so that the library interface is informative and visually appealing.

#### Acceptance Criteria

1. WHEN a MediaItem is created THEN the System SHALL optionally query external metadata providers for overview, genres, cast, runtime, and poster URL
2. WHEN external metadata is fetched THEN the System SHALL cache the results in the database to avoid repeated API calls
3. IF an external metadata provider is unreachable THEN the System SHALL gracefully degrade and use parsed metadata without blocking import
4. WHEN the user requests metadata refresh THEN the System SHALL re-query external providers and update cached metadata
5. WHERE metadata includes poster or backdrop URLs THEN the System SHALL download and cache images locally for offline access
6. WHEN displaying MediaItems THEN the System SHALL use cached metadata and images without requiring network access

### Requirement 14: Audit Logging and Traceability

**User Story:** As a user, I want a complete record of all file operations, so that I can trace changes, debug issues, and recover from errors.

#### Acceptance Criteria

1. WHEN a file operation occurs THEN the System SHALL create an Audit Log entry with operation type, timestamp, affected file path, old status, new status, and initiator (user or system job)
2. WHEN a FileVersion is moved THEN the System SHALL log both the old path and new path
3. WHEN a FileVersion is deleted THEN the System SHALL log the deletion with file size and reason
4. WHEN the user views audit logs THEN the System SHALL display entries in reverse chronological order with filtering by operation type, date range, and file path
5. WHERE an operation fails THEN the System SHALL log the error details and stack trace for debugging
6. WHEN audit logs exceed a configured size threshold THEN the System SHALL archive old entries to prevent unbounded growth

### Requirement 15: Configuration and Policy Management

**User Story:** As a user, I want to configure library behavior and policies, so that the system adapts to my hardware constraints and preferences.

#### Acceptance Criteria

1. WHEN the user accesses library settings THEN the System SHALL display configurable options for trash grace period, hashing concurrency, rescan batch size, and cleanup thresholds
2. WHEN the user sets a quality preference policy THEN the System SHALL apply the policy when recommending preferred versions (e.g., prefer highest resolution, prefer smallest size, prefer specific codec)
3. WHEN the user sets a version retention policy THEN the System SHALL configure how many versions to keep per MediaItem (e.g., keep best only, keep best plus one backup, keep all)
4. WHEN the user modifies settings THEN the System SHALL validate the configuration and persist changes to the database
5. WHERE settings affect background jobs THEN the System SHALL apply new settings to subsequently scheduled jobs without requiring restart
6. WHEN the user resets settings THEN the System SHALL restore default values appropriate for HDD-based storage and modest hardware

### Requirement 16: Watch Folder and Automatic Import

**User Story:** As a user, I want directories to be automatically monitored for new files, so that downloads are imported without manual intervention.

#### Acceptance Criteria

1. WHEN the user designates a directory as a Watch Folder THEN the System SHALL monitor that directory for new files using Linux inotify events
2. WHEN a new file appears in a Watch Folder THEN the System SHALL wait for file write completion before triggering import
3. WHEN a file write is detected as complete THEN the System SHALL automatically move the file to the Staging Directory and begin metadata probing
4. WHERE a Watch Folder contains subdirectories THEN the System SHALL recursively monitor all subdirectories for new files
5. WHEN a Watch Folder is added THEN the System SHALL optionally perform an initial scan to import existing files
6. IF a file in a Watch Folder fails import validation THEN the System SHALL move the file to a failed imports directory and log the error
7. WHEN the user disables a Watch Folder THEN the System SHALL stop monitoring that directory without affecting existing library content

### Requirement 17: Symbolic Link and Hard Link Support

**User Story:** As a user, I want to use symbolic and hard links for space-efficient organization, so that I can reference files in multiple locations without duplication.

#### Acceptance Criteria

1. WHEN the System encounters a symbolic link during scanning THEN the System SHALL resolve the link target and use the target path for FileVersion records
2. WHEN a FileVersion is created from a symbolic link THEN the System SHALL store both the link path and resolved target path
3. WHEN the user creates a hard link to a library file THEN the System SHALL detect the shared inode and associate both paths with the same FileVersion
4. WHERE a symbolic link target does not exist THEN the System SHALL mark the FileVersion as missing and log a broken link warning
5. WHEN displaying FileVersion information THEN the System SHALL indicate whether the path is a symbolic link, hard link, or regular file
6. WHEN the user requests to follow symbolic links during rescan THEN the System SHALL provide a configuration option to enable or disable link following

### Requirement 18: Linux Filesystem Compatibility

**User Story:** As a user on Linux, I want full compatibility with Linux filesystems and conventions, so that the system works reliably with ext4, btrfs, xfs, and other Linux filesystems.

#### Acceptance Criteria

1. WHEN performing file operations THEN the System SHALL use POSIX-compliant system calls for maximum compatibility across Linux filesystems
2. WHEN handling file paths THEN the System SHALL correctly process Linux path separators, hidden files (dot-prefixed), and case-sensitive filenames
3. WHEN checking file permissions THEN the System SHALL respect Linux user, group, and other permission bits
4. WHERE files are stored on ext4, btrfs, or xfs filesystems THEN the System SHALL utilize filesystem-specific features like reflinks (btrfs, xfs) for efficient copying when available
5. WHEN atomic operations are required THEN the System SHALL use rename system calls within the same filesystem to ensure atomicity
6. WHEN the System detects filesystem mount points THEN the System SHALL avoid moving files across mount points and instead use copy-then-delete operations
7. WHERE extended attributes are available THEN the System SHALL optionally store custom metadata in xattrs for filesystem-level tagging

### Requirement 19: Collections and Smart Collections

**User Story:** As a user, I want to organize media into collections and create dynamic smart collections, so that I can group related content and automatically categorize media based on criteria.

#### Acceptance Criteria

1. WHEN the user creates a Collection THEN the System SHALL store the collection name, description, and creation timestamp
2. WHEN the user adds MediaItems to a Collection THEN the System SHALL create associations between the Collection and selected MediaItems
3. WHEN the user views a Collection THEN the System SHALL display all associated MediaItems with standard library browsing features
4. WHEN the user creates a Smart Collection THEN the System SHALL store filter criteria including title patterns, year ranges, quality labels, tags, ratings, and version counts
5. WHEN a Smart Collection is viewed THEN the System SHALL dynamically query MediaItems matching the stored criteria
6. WHEN a new MediaItem is added to the library THEN the System SHALL automatically update Smart Collections whose criteria match the new item
7. WHERE a MediaItem matches multiple Smart Collections THEN the System SHALL include it in all matching collections
8. WHEN the user exports a Collection THEN the System SHALL generate a file list with paths and metadata for all contained MediaItems

### Requirement 20: Advanced Search and Filtering

**User Story:** As a user, I want powerful search and filtering capabilities, so that I can quickly locate specific media in large libraries.

#### Acceptance Criteria

1. WHEN the user performs a text search THEN the System SHALL search across title, original title, alternative titles, cast names, director names, and custom notes
2. WHEN the user applies multiple filters THEN the System SHALL combine filters with AND logic and return MediaItems matching all criteria
3. WHEN the user creates a saved search THEN the System SHALL store the search query and filter combination for quick reuse
4. WHERE the user searches by file properties THEN the System SHALL support filtering by file size ranges, codec types, audio track languages, subtitle languages, and container formats
5. WHEN the user searches by date THEN the System SHALL support filtering by added date, release year, last modified date, and last watched date
6. WHEN the user performs a fuzzy search THEN the System SHALL return results with approximate title matches using Levenshtein distance or similar algorithms
7. WHEN search results are displayed THEN the System SHALL highlight matching terms in titles and metadata

### Requirement 21: Batch Operations and Bulk Editing

**User Story:** As a user, I want to perform operations on multiple media items simultaneously, so that I can efficiently manage large selections.

#### Acceptance Criteria

1. WHEN the user selects multiple MediaItems THEN the System SHALL enable batch operation controls for tagging, rating, collection assignment, and deletion
2. WHEN the user applies tags to a batch THEN the System SHALL add the specified tags to all selected MediaItems
3. WHEN the user assigns a rating to a batch THEN the System SHALL update the user rating for all selected MediaItems
4. WHEN the user moves a batch to a Collection THEN the System SHALL create associations for all selected MediaItems
5. WHERE the user deletes a batch THEN the System SHALL soft-delete all FileVersions associated with selected MediaItems and create Audit Log entries for each
6. WHEN a batch operation is in progress THEN the System SHALL display progress with item count and percentage completion
7. IF any item in a batch operation fails THEN the System SHALL log the error, skip that item, and continue processing remaining items

### Requirement 22: Export and Backup

**User Story:** As a user, I want to export library metadata and create backups, so that I can recover from data loss or migrate to another system.

#### Acceptance Criteria

1. WHEN the user requests a metadata export THEN the System SHALL generate a JSON or CSV file containing all MediaItem and FileVersion records with full metadata
2. WHEN the user exports the library THEN the System SHALL include Collections, Smart Collections, user tags, ratings, notes, and audit logs in the export
3. WHEN the user creates a backup THEN the System SHALL export the SQLite database file and optionally include cached artwork and metadata
4. WHERE the user specifies an Export Profile THEN the System SHALL apply filters to export only selected MediaItems, Collections, or date ranges
5. WHEN the user imports a metadata export THEN the System SHALL merge imported records with existing library data and resolve conflicts based on user preference
6. WHEN the user schedules automatic backups THEN the System SHALL create periodic exports at configured intervals and retain a specified number of backup versions
7. WHERE backup files exceed a size threshold THEN the System SHALL compress exports using gzip or similar compression

### Requirement 23: File Integrity Verification

**User Story:** As a user, I want to verify file integrity over time, so that I can detect bit rot, corruption, or silent data degradation on HDDs.

#### Acceptance Criteria

1. WHEN a FileVersion is committed THEN the System SHALL optionally compute and store a cryptographic checksum (SHA-256 or BLAKE3)
2. WHEN the user requests integrity verification THEN the System SHALL recompute checksums for selected FileVersions and compare with stored values
3. IF a checksum mismatch is detected THEN the System SHALL mark the FileVersion as corrupted and create an alert notification
4. WHEN a corrupted file is detected THEN the System SHALL log the corruption event with file path, expected checksum, actual checksum, and detection timestamp
5. WHERE multiple versions of a MediaItem exist THEN the System SHALL suggest replacing corrupted versions with intact versions
6. WHEN the user schedules periodic integrity checks THEN the System SHALL verify a configurable percentage of the library on each run to distribute I/O load
7. WHILE integrity verification runs THEN the System SHALL throttle I/O operations to avoid impacting system performance

### Requirement 24: Metadata Editing and Correction

**User Story:** As a user, I want to manually edit and correct metadata, so that I can fix errors from automatic parsing or external providers.

#### Acceptance Criteria

1. WHEN the user views MediaItem details THEN the System SHALL display editable fields for title, original title, year, overview, genres, cast, director, and runtime
2. WHEN the user modifies metadata THEN the System SHALL mark the field as user-edited to prevent automatic overwrites during metadata refresh
3. WHEN the user resets a field THEN the System SHALL restore the original value from external providers or parsed metadata
4. WHERE the user edits technical metadata THEN the System SHALL allow modification of resolution, codec, quality label, and audio/subtitle track information
5. WHEN the user adds alternative titles THEN the System SHALL store multiple title variants for improved search matching
6. WHEN the user uploads custom artwork THEN the System SHALL store the image locally and use it instead of provider-sourced posters
7. WHERE metadata conflicts exist between FileVersions THEN the System SHALL display all variants and allow the user to select the canonical version

### Requirement 25: Playback Integration and History

**User Story:** As a user, I want to track playback history and integrate with media players, so that I can resume watching and see what I've watched.

#### Acceptance Criteria

1. WHEN the user plays a FileVersion THEN the System SHALL record a playback event with timestamp, file path, and playback duration
2. WHEN the user stops playback THEN the System SHALL store the playback position for resume functionality
3. WHEN the user views a MediaItem THEN the System SHALL display watch status (unwatched, in progress, watched) and last watched date
4. WHERE a MediaItem has multiple FileVersions THEN the System SHALL track playback history separately for each version
5. WHEN the user marks a MediaItem as watched THEN the System SHALL update the watch status without requiring actual playback
6. WHEN the user opens a FileVersion for playback THEN the System SHALL launch the configured default media player with the file path
7. WHERE playback position is stored THEN the System SHALL provide an option to resume from the last position or start from the beginning

### Requirement 26: Notification and Alert System

**User Story:** As a user, I want to receive notifications about important library events, so that I can respond to issues and stay informed about background operations.

#### Acceptance Criteria

1. WHEN a background job completes THEN the System SHALL create a notification with job type, status, and summary results
2. WHEN a file integrity check detects corruption THEN the System SHALL create a high-priority alert notification
3. WHEN disk space falls below a configured threshold THEN the System SHALL create a warning notification with current usage and cleanup suggestions
4. WHERE a rescan detects missing files THEN the System SHALL create a notification listing the count of missing files and affected MediaItems
5. WHEN a Watch Folder import fails THEN the System SHALL create a notification with the failed file path and error reason
6. WHEN the user views notifications THEN the System SHALL display a list of unread notifications with timestamps and severity levels
7. WHERE notifications accumulate THEN the System SHALL automatically dismiss notifications older than a configured retention period

### Requirement 27: Performance Optimization for HDD

**User Story:** As a user with HDD-based storage, I want the system optimized for sequential access and minimal seeks, so that performance remains acceptable on mechanical drives.

#### Acceptance Criteria

1. WHEN the System performs file operations THEN the System SHALL batch operations to minimize random seeks and maximize sequential I/O
2. WHEN scanning directories THEN the System SHALL read directory entries in filesystem order to leverage HDD read-ahead caching
3. WHERE multiple files require processing THEN the System SHALL sort operations by physical disk location when possible to reduce seek time
4. WHEN computing hashes or checksums THEN the System SHALL use large read buffers (1MB or greater) to optimize sequential read performance
5. WHEN the System writes files THEN the System SHALL use buffered I/O with appropriate buffer sizes for HDD characteristics
6. WHERE the System detects HDD storage THEN the System SHALL automatically adjust I/O concurrency limits to prevent thrashing
7. WHEN the user configures performance settings THEN the System SHALL provide HDD-optimized presets for buffer sizes, concurrency, and batch sizes

### Requirement 28: Import Profiles and Automation Rules

**User Story:** As a user, I want to define import profiles with automation rules, so that different types of content are processed according to specific preferences.

#### Acceptance Criteria

1. WHEN the user creates an Import Profile THEN the System SHALL store rules for file naming patterns, quality preferences, automatic tagging, and collection assignment
2. WHEN a file matches an Import Profile pattern THEN the System SHALL automatically apply the profile's rules during import
3. WHERE an Import Profile specifies automatic tagging THEN the System SHALL add configured tags to imported MediaItems
4. WHEN an Import Profile includes quality filtering THEN the System SHALL reject files below specified quality thresholds
5. WHERE an Import Profile specifies a target Collection THEN the System SHALL automatically add imported MediaItems to that Collection
6. WHEN multiple Import Profiles match a file THEN the System SHALL apply the profile with the highest priority or most specific pattern
7. WHEN the user tests an Import Profile THEN the System SHALL show a preview of how the profile would process sample files without committing changes

### Requirement 29: Database Maintenance and Optimization

**User Story:** As a user, I want the database to remain performant as the library grows, so that queries and operations stay fast even with thousands of media items.

#### Acceptance Criteria

1. WHEN the database size exceeds a threshold THEN the System SHALL automatically run VACUUM operations during idle periods to reclaim space
2. WHEN query performance degrades THEN the System SHALL analyze and rebuild database indexes to optimize query plans
3. WHERE the database contains orphaned records THEN the System SHALL identify and optionally remove records not associated with any MediaItem or FileVersion
4. WHEN the user requests database statistics THEN the System SHALL display table sizes, index sizes, record counts, and fragmentation levels
5. WHEN the System detects database corruption THEN the System SHALL attempt automatic recovery using SQLite integrity checks and backups
6. WHERE the database file grows large THEN the System SHALL provide an option to archive old audit logs and playback history to a separate database
7. WHEN database maintenance operations run THEN the System SHALL ensure WAL mode is enabled for concurrent access during maintenance

### Requirement 30: Linux Desktop Integration

**User Story:** As a Linux desktop user, I want integration with desktop environments and system features, so that the application feels native and leverages platform capabilities.

#### Acceptance Criteria

1. WHEN the application starts THEN the System SHALL register with the desktop environment for file associations and MIME type handling
2. WHEN the user right-clicks a media file in a file manager THEN the System SHALL appear in the context menu with options to import or add to library
3. WHERE the desktop environment supports notifications THEN the System SHALL use native notification APIs (libnotify, D-Bus) for alerts
4. WHEN the user drags files from a file manager THEN the System SHALL accept drag-and-drop operations for import
5. WHERE the desktop environment provides a system tray THEN the System SHALL display a tray icon with quick access to library functions and job status
6. WHEN the System performs intensive operations THEN the System SHALL respect system power management settings and pause during suspend/hibernate
7. WHERE the desktop environment supports thumbnails THEN the System SHALL generate and cache video thumbnails for file manager integration
