# User Guide: Import Workflow

## Overview

The import workflow in TorrentFlix provides a safe, multi-stage process for adding media files to your library. Files are staged, validated, and committed atomically to ensure data integrity.

## Table of Contents

1. [Import Process Overview](#import-process-overview)
2. [Manual Import](#manual-import)
3. [Watch Folders](#watch-folders)
4. [Import Profiles](#import-profiles)
5. [Staging and Validation](#staging-and-validation)
6. [Committing Files](#committing-files)
7. [Error Handling](#error-handling)
8. [Troubleshooting](#troubleshooting)

---

## Import Process Overview

The import workflow consists of these stages:

```
File Selection
    ↓
Staging (copy to temporary location)
    ↓
Metadata Probing (extract technical details)
    ↓
Fingerprinting (compute file hash)
    ↓
Commit (move to library root)
    ↓
Database Update (create MediaItem and FileVersion)
```

**Key Benefits:**
- **Safety**: Files are validated before being added to the library
- **Atomicity**: Either the entire import succeeds or fails cleanly
- **Recoverability**: Failed imports remain in staging for retry
- **Efficiency**: Metadata is extracted once and cached

---

## Manual Import

### Importing Individual Files

To manually import a file:

1. Click "Import" in the main menu
2. Select "Import File"
3. Choose a file from your filesystem
4. Select the target library root
5. Click "Import"

**What happens:**
- File is copied to staging directory
- Metadata is probed (resolution, codec, duration, etc.)
- File hash is computed for duplicate detection
- You're prompted to confirm the import

### Importing Multiple Files

To import multiple files at once:

1. Click "Import" in the main menu
2. Select "Import Folder"
3. Choose a directory
4. Select import options:
   - **Recursive**: Include subdirectories
   - **Apply Profile**: Use import profile rules
   - **Target Root**: Select library root
5. Click "Import"

**Progress:**
- Files are processed sequentially
- Progress bar shows current file and overall progress
- You can pause or cancel the import

### Drag and Drop

Drag files or folders directly into the library view:

1. Select files in your file manager
2. Drag them into the TorrentFlix window
3. Drop to start import
4. Confirm import options

---

## Watch Folders

Watch folders automatically import new files as they appear.

### Setting Up a Watch Folder

1. Click "Settings" → "Watch Folders"
2. Click "Add Watch Folder"
3. Select a directory to monitor
4. Configure options:
   - **Recursive**: Monitor subdirectories
   - **Import Profile**: Apply rules to imported files
   - **Initial Scan**: Import existing files
5. Click "Save"

**What happens:**
- Directory is monitored for new files
- When a file appears and finishes writing, it's automatically imported
- Failed imports are moved to a "failed imports" directory

### Managing Watch Folders

**To disable a watch folder:**
1. Click "Settings" → "Watch Folders"
2. Find the folder in the list
3. Click "Disable"
4. Monitoring stops, but existing library content is unaffected

**To remove a watch folder:**
1. Click "Settings" → "Watch Folders"
2. Find the folder in the list
3. Click "Remove"
4. Monitoring stops

### Write Completion Detection

The system detects when a file has finished writing:

- **Timeout-based**: Waits for file to stop changing for 5 seconds
- **Size-based**: Detects when file size stabilizes
- **Automatic retry**: Retries if file is still being written

**Typical scenarios:**
- Download completes → file is imported
- Torrent finishes → file is imported
- Manual file copy → file is imported after copy completes

---

## Import Profiles

Import profiles automate file processing with predefined rules.

### Creating an Import Profile

1. Click "Settings" → "Import Profiles"
2. Click "New Profile"
3. Enter profile details:
   - **Name**: Descriptive name (e.g., "4K Movies")
   - **Description**: Optional notes
   - **Priority**: Higher priority profiles are checked first
4. Set matching rules:
   - **File Pattern**: Glob or regex pattern (e.g., `*.mkv`, `*4K*`)
5. Set quality filters:
   - **Minimum Resolution**: Reject files below this resolution
   - **Allowed Codecs**: Only accept specific codecs
6. Set automation rules:
   - **Auto-Tags**: Tags to apply automatically
   - **Target Collection**: Collection to add to
   - **Target Library Root**: Where to store files
7. Click "Save"

### Using Import Profiles

**Manual import with profile:**
1. Click "Import" → "Import File"
2. Select a file
3. Choose "Apply Profile" option
4. Select the profile
5. Click "Import"

**Watch folder with profile:**
1. Create a watch folder
2. Select an import profile
3. Files matching the profile are automatically processed

### Profile Matching

When multiple profiles match a file:

- **Priority**: Higher priority profile is used
- **First Match**: First matching profile in priority order
- **No Match**: File is imported without profile rules

**Example profiles:**
- "4K Movies": Pattern `*4K*`, min resolution 2160p, auto-tag "4K"
- "BluRay": Pattern `*BluRay*`, auto-tag "BluRay", target collection "BluRay Collection"
- "Archive": Pattern `*`, target root "Archive Storage"

---

## Staging and Validation

### Understanding Staging

Files are temporarily stored in a staging directory during import:

- **Location**: `.data/staging/` (configurable)
- **Purpose**: Validate files before adding to library
- **Retention**: Failed imports remain for retry
- **Cleanup**: Successful imports are removed after commit

### Metadata Probing

During staging, the system extracts technical metadata:

**Video Information:**
- Resolution (width × height)
- Codec (H.264, H.265, VP9, etc.)
- Bitrate
- Frame rate
- Color space

**Audio Information:**
- Codec (AAC, AC3, FLAC, etc.)
- Bitrate
- Sample rate
- Language
- Number of tracks

**Subtitle Information:**
- Language
- Format (SRT, ASS, PGS, etc.)
- Number of tracks

**Container Information:**
- Format (MP4, MKV, AVI, etc.)
- Duration
- Overall bitrate

### Fingerprinting

Two types of fingerprints are computed:

**Fast Hash:**
- First 1MB of file
- Last 1MB of file
- File size
- Used for quick duplicate detection
- Computed during staging

**Full Hash:**
- SHA-256 or BLAKE3 of entire file
- Used for exact duplicate detection
- Computed in background after import
- Takes longer but more reliable

### Viewing Staged Files

To see files waiting to be imported:

1. Click "Import" → "View Staged Files"
2. See list of files in staging:
   - File name and size
   - Metadata status (pending, ready, error)
   - Matched profile (if any)
   - Error message (if failed)

---

## Committing Files

### Confirming Import

After staging and validation, confirm the import:

1. Review the staged file details
2. Verify metadata was extracted correctly
3. Check that the correct library root is selected
4. Click "Commit" to finalize

**What happens:**
- File is moved from staging to library root
- MediaItem is created or updated
- FileVersion record is created
- File is removed from staging
- Database is updated atomically

### Atomic Commit

The commit process is atomic:

- **All or Nothing**: Either the entire operation succeeds or fails
- **Transaction**: Database changes are wrapped in a transaction
- **Rollback**: If anything fails, all changes are rolled back
- **File Safety**: Files are only moved after database is updated

### Naming Scheme

Files are organized in the library root using a consistent naming scheme:

```
Library Root/
├── Title (Year)/
│   ├── Title (Year) - 1080p.mkv
│   ├── Title (Year) - 720p.mkv
│   └── Title (Year) - 4K.mkv
└── Another Title (2023)/
    └── Another Title (2023) - BluRay.mkv
```

**Benefits:**
- Consistent organization
- Easy to browse on disk
- Supports multiple versions per media
- Handles special characters safely

---

## Error Handling

### Common Import Errors

**File Not Found:**
- File was deleted before import completed
- Check that file still exists
- Retry the import

**Insufficient Disk Space:**
- Not enough space in staging or library root
- Free up disk space
- Retry the import

**Permission Denied:**
- Insufficient permissions to read source file
- Insufficient permissions to write to library root
- Check file and directory permissions

**Metadata Extraction Failed:**
- File is corrupted or not a valid media file
- Try with a different file
- Check file format is supported

**Database Error:**
- Database is locked or corrupted
- Restart the application
- Check database integrity

### Retry Failed Imports

To retry a failed import:

1. Click "Import" → "View Staged Files"
2. Find the failed file
3. Click "Retry"
4. Fix any issues (e.g., free up disk space)
5. Click "Retry" again

### Failed Imports Directory

Files that fail import are moved to a failed imports directory:

- **Location**: `.data/failed_imports/`
- **Purpose**: Preserve failed files for investigation
- **Cleanup**: Manually delete when no longer needed

---

## Troubleshooting

### Import Hangs or Freezes

**Symptoms:**
- Import progress bar stops moving
- Application becomes unresponsive

**Solutions:**
1. Check if file is still being written (watch folder scenario)
2. Check disk space availability
3. Try canceling and retrying
4. Restart the application

### Metadata Not Extracted

**Symptoms:**
- Staged file shows "pending" status
- Metadata fields are empty

**Solutions:**
1. Check that ffprobe is installed and working
2. Verify file is a valid media file
3. Try with a different file
4. Check application logs for errors

### Files Not Imported from Watch Folder

**Symptoms:**
- Files appear in watch folder but aren't imported
- No error messages

**Solutions:**
1. Check that watch folder is enabled
2. Verify file write has completed (wait a few seconds)
3. Check file permissions
4. Check disk space
5. Review application logs

### Duplicate Files Not Detected

**Symptoms:**
- Same file imported multiple times
- No duplicate warning

**Solutions:**
1. Full hash computation may still be in progress
2. Check "Storage Analytics" for duplicates
3. Run cleanup to identify and remove duplicates
4. Check that fast hash was computed during staging

### Import Profile Not Applied

**Symptoms:**
- File imported but profile rules not applied
- Tags not added, collection not updated

**Solutions:**
1. Check profile is enabled
2. Verify file matches profile pattern
3. Check profile priority (higher priority wins)
4. Review profile configuration

---

## Best Practices

### Organization

- **Use consistent file naming**: Helps with metadata matching
- **Group by type**: Separate movies and TV series
- **Use watch folders**: Automate imports from download directory
- **Create profiles**: Standardize import rules

### Performance

- **Import during off-hours**: Large imports can impact performance
- **Use profiles**: Reduces manual configuration
- **Monitor staging**: Keep staging directory clean
- **Regular cleanup**: Remove failed imports

### Safety

- **Verify metadata**: Check extracted metadata is correct
- **Use preferred versions**: Mark your best quality files
- **Backup important files**: Keep copies of critical media
- **Monitor disk space**: Prevent running out of space

### Maintenance

- **Review failed imports**: Investigate why imports failed
- **Clean staging directory**: Remove old staged files
- **Check watch folders**: Ensure they're still needed
- **Update profiles**: Adjust rules as needed

---

## Advanced Topics

### Custom File Patterns

Import profiles support glob and regex patterns:

**Glob patterns:**
- `*.mkv` - All MKV files
- `*4K*` - Files containing "4K"
- `*BluRay*` - Files containing "BluRay"
- `[0-9]*.mp4` - MP4 files starting with number

**Regex patterns:**
- `.*4K.*` - Files containing "4K"
- `.*\(2023\).*` - Files containing "(2023)"
- `^[A-Z].*` - Files starting with uppercase letter

### Quality Filtering

Profiles can enforce quality requirements:

- **Minimum Resolution**: Reject files below threshold (e.g., 1080p)
- **Allowed Codecs**: Only accept specific codecs (e.g., H.265, VP9)
- **Minimum Bitrate**: Reject low-bitrate files

**Example:**
- Profile "4K Only": Min resolution 2160p, allowed codecs H.265
- Profile "Efficient": Allowed codecs H.265, min bitrate 5000 kbps

### Batch Import Configuration

For large imports, configure batch settings:

1. Click "Settings" → "Import"
2. Set "Batch Size": Number of files to process in parallel
3. Set "Batch Timeout": Maximum time per batch
4. Click "Save"

**Recommendations:**
- Batch size 1-5 for HDD storage
- Batch size 5-10 for SSD storage
- Adjust based on available disk I/O

---

## Getting Help

- **Import Logs**: Check application logs for detailed error messages
- **Staging Directory**: Inspect staged files to debug issues
- **Failed Imports**: Review failed imports directory for clues
- **Support**: Report issues with specific file types or patterns

