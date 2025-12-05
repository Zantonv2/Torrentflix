# User Guide: Library Management Features

## Overview

The Library Management system in TorrentFlix provides comprehensive tools for organizing, browsing, and managing your media collection. This guide covers the core features you'll use daily.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Browsing Your Library](#browsing-your-library)
3. [Searching and Filtering](#searching-and-filtering)
4. [Managing Media Items](#managing-media-items)
5. [Working with Versions](#working-with-versions)
6. [User Metadata](#user-metadata)
7. [Collections](#collections)
8. [Storage Management](#storage-management)

---

## Getting Started

### Setting Up Library Roots

Before you can import media, you need to designate one or more directories as **Library Roots** where your media files will be stored.

**To add a library root:**

1. Open Settings → Library Roots
2. Click "Add Library Root"
3. Select a directory on your storage device
4. Give it a descriptive name (e.g., "Main HDD", "Archive Storage")
5. Click "Save"

**Tips:**
- Use separate roots for different storage devices (e.g., one for your main HDD, one for external drives)
- Mark slower storage as "Archive" to deprioritize background operations
- Ensure you have write permissions to the directory

### Understanding Media Items and Versions

The library organizes media into two levels:

- **MediaItem**: A logical entity representing a unique movie or TV series (identified by title and year)
- **FileVersion**: A physical file on disk associated with a MediaItem

For example, you might have one MediaItem for "The Matrix (1999)" with three FileVersions:
- 1080p BluRay x264 (2.5 GB)
- 720p WEB-DL x264 (1.2 GB)
- 4K UHD BluRay x265 (15 GB)

---

## Browsing Your Library

### Library View

The main library view displays all your MediaItems in a grid or list format.

**Features:**
- **Grid View**: Shows poster thumbnails with title and year
- **List View**: Shows detailed information in rows
- **Version Count**: Displays how many versions you have for each item
- **Total Size**: Shows combined size of all versions
- **Lazy Loading**: Posters load as you scroll for better performance

### Viewing Media Details

Click on any MediaItem to open the detail panel, which shows:

- **Metadata**: Title, year, overview, genres, cast, director, runtime
- **Ratings**: User rating, external ratings (IMDb, TMDB, Kinopoisk)
- **Versions**: All FileVersions with technical details
- **Watch Status**: Whether you've watched it and when
- **Custom Notes**: Your personal notes about the media
- **Tags**: User-defined tags for organization

---

## Searching and Filtering

### Quick Search

Use the search bar at the top to find media by title:

- **Case-insensitive**: "the matrix" finds "The Matrix"
- **Partial matching**: "matrix" finds "The Matrix", "Matrix Reloaded", etc.
- **Fuzzy matching**: Typos are tolerated (e.g., "matirx" finds "The Matrix")

### Advanced Filtering

Click "Filters" to access advanced search options:

**Available Filters:**
- **Title**: Exact or partial title match
- **Year**: Specific year or range (e.g., 1990-2000)
- **Resolution**: 4K, 1080p, 720p, SD
- **Quality Label**: BluRay, WEB-DL, HDTV, DVD, etc.
- **Status**: Present, Missing, Trashed, Corrupted
- **Tags**: Filter by one or more tags
- **Rating**: Minimum user rating (0.0-10.0)
- **Watch Status**: Unwatched, In Progress, Watched

**Combining Filters:**
- Multiple filters use AND logic (all must match)
- Select multiple values within a filter to use OR logic (any can match)

### Saved Searches

Save frequently-used search combinations for quick access:

1. Set up your filters
2. Click "Save Search"
3. Give it a name (e.g., "Unwatched 1080p")
4. Click "Save"

Access saved searches from the search bar dropdown.

---

## Managing Media Items

### Editing Metadata

You can manually edit metadata for any MediaItem:

1. Open the media detail panel
2. Click "Edit" next to any metadata field
3. Make your changes
4. Click "Save"

**Important:** Edited fields are marked to prevent automatic overwrites during metadata refresh.

### Resetting Metadata

To restore original metadata from external providers:

1. Open the media detail panel
2. Click the "Reset" button next to an edited field
3. Confirm the reset

### Adding Custom Notes

Add personal notes to any MediaItem:

1. Open the media detail panel
2. Click in the "Custom Notes" field
3. Type your notes
4. Changes are saved automatically

**Use cases:**
- Personal reviews or recommendations
- Viewing notes (e.g., "Watched with family")
- Quality assessments
- Reminders about content

### Deleting Media Items

To delete a MediaItem and all its versions:

1. Open the media detail panel
2. Click "Delete MediaItem"
3. Confirm the deletion

**Note:** This performs a soft-delete, moving files to trash. You can recover them within the grace period.

---

## Working with Versions

### Understanding Version Information

Each FileVersion displays:

- **Path**: Location on disk
- **File Size**: Size in GB/MB
- **Resolution**: Video resolution (e.g., 1920x1080)
- **Codec**: Video codec (e.g., H.264, H.265)
- **Container**: File format (e.g., MP4, MKV)
- **Quality Label**: Parsed quality classification
- **Status**: Present, Missing, Trashed, or Corrupted
- **Added Date**: When the file was imported
- **Preferred Flag**: Whether this is your preferred version

### Setting a Preferred Version

Mark a version as preferred to prioritize it for playback and cleanup:

1. Open the media detail panel
2. Find the version you want to prefer
3. Click the "Star" icon to mark as preferred
4. Only one version can be preferred per MediaItem

**Impact:**
- Preferred versions are highlighted in the UI
- Preferred versions are protected during cleanup
- Playback defaults to the preferred version

### Comparing Versions

View a side-by-side comparison of all versions:

1. Open the media detail panel
2. Click "Compare Versions"
3. View quality scores and technical details
4. The highest-scoring version is recommended

**Quality Score Factors:**
- Resolution (4K > 1080p > 720p > SD)
- Codec (H.265 > H.264 > others)
- Source (BluRay > WEB-DL > HDTV > DVD)
- File size (reasonable sizes preferred, extremes penalized)

### Identifying Duplicates

The system automatically identifies exact duplicates (same file hash):

1. Open the media detail panel
2. Look for "Duplicate Versions" section
3. Identical files are grouped together
4. Consider deleting lower-quality duplicates

---

## User Metadata

### User Ratings

Rate media items on a scale of 0.0 to 10.0:

1. Open the media detail panel
2. Click on the rating field
3. Enter your rating (0-10)
4. Press Enter to save

**Display:**
- Your rating appears alongside external ratings
- Ratings are used in filtering and sorting

### Tags

Organize media with custom tags:

**Adding Tags:**
1. Open the media detail panel
2. Click "Add Tag"
3. Select existing tags or create new ones
4. Click "Save"

**Managing Tags:**
- Create tags for genres, moods, or categories
- Assign multiple tags to one item
- Filter library by tags

**Example Tags:**
- Genres: Action, Comedy, Drama, Horror
- Moods: Feel-good, Intense, Relaxing
- Status: Favorites, To-watch, Rewatchable

### Watch Status

Track your viewing progress:

**Options:**
- **Unwatched**: Haven't started
- **In Progress**: Currently watching
- **Watched**: Completed

**Setting Watch Status:**
1. Open the media detail panel
2. Click on the watch status dropdown
3. Select the appropriate status
4. Status is saved automatically

**Display:**
- Watch status appears in library view
- Last watched date is recorded
- Used in filtering and sorting

---

## Collections

Collections let you group related media items together.

### Manual Collections

Create custom groupings:

1. Click "Collections" in the sidebar
2. Click "New Collection"
3. Enter a name and optional description
4. Click "Create"
5. Add media items by clicking "Add to Collection"

**Use cases:**
- Movie series (e.g., "Marvel Cinematic Universe")
- Themed collections (e.g., "Best of 2023")
- Personal lists (e.g., "Favorites", "To-watch")

### Smart Collections

Create dynamic collections that automatically update:

1. Click "Collections" in the sidebar
2. Click "New Smart Collection"
3. Enter a name
4. Set filter criteria:
   - Title patterns
   - Year ranges
   - Quality labels
   - Tags
   - Ratings
   - Version counts
5. Click "Create"

**Automatic Updates:**
- New media matching the criteria are automatically added
- Media no longer matching are automatically removed
- Updates happen in real-time

**Example Smart Collections:**
- "4K Movies" (resolution = 4K)
- "Unwatched Favorites" (watch status = unwatched AND rating >= 8.0)
- "Recent Additions" (added date within last 30 days)

### Exporting Collections

Export a collection as a file list:

1. Open the collection
2. Click "Export"
3. Choose format (JSON or CSV)
4. Select what to include:
   - File paths
   - Metadata
   - Ratings and tags
5. Click "Export"

**Use cases:**
- Share with others
- Backup collection data
- Import into other applications

---

## Storage Management

### Viewing Storage Analytics

Monitor how your disk space is used:

1. Click "Storage Analytics" in the sidebar
2. View statistics:
   - Total library size
   - Number of MediaItems and FileVersions
   - Average versions per media
   - Breakdown by resolution, quality, and status

### Identifying Duplicates and Variants

Find opportunities to save space:

1. Click "Storage Analytics"
2. Scroll to "Duplicate Analysis"
3. View:
   - Exact duplicates (same file hash)
   - Variants (same MediaItem, different versions)
   - Wasted space from duplicates

### Cleanup Candidates

The system identifies files that can be safely deleted:

1. Click "Cleanup" in the sidebar
2. View candidates:
   - Exact duplicates (keep one, delete others)
   - Lower-quality variants
   - Trashed files past grace period
   - Missing file records
3. Review the list and select items to delete
4. View estimated space savings
5. Click "Execute Cleanup" to delete selected items

**Safety Features:**
- Preferred versions are protected
- Soft-delete allows recovery within grace period
- Errors are logged and don't stop the process

### Disk Space Monitoring

The system monitors available disk space:

- **Threshold Alert**: Notified when free space drops below threshold
- **Cleanup Suggestions**: Recommended cleanup candidates
- **Automatic Notifications**: Desktop notifications when space is low

**To adjust threshold:**
1. Open Settings → Storage
2. Set "Disk Space Warning Threshold"
3. Click "Save"

---

## Tips and Best Practices

### Organization

- **Use consistent naming**: Helps with metadata matching
- **Create meaningful collections**: Makes browsing easier
- **Tag strategically**: Use tags for quick filtering
- **Set preferred versions**: Protects your best quality files

### Performance

- **Regular rescans**: Keep database in sync with filesystem
- **Periodic cleanup**: Remove duplicates and old trashed files
- **Archive old storage**: Mark slow drives as archive
- **Monitor disk space**: Prevent running out of space

### Maintenance

- **Check integrity**: Periodically verify file integrity
- **Review audit logs**: Track what's been changed
- **Backup collections**: Export important collections
- **Update metadata**: Refresh metadata for new releases

### Troubleshooting

**Missing Files:**
- Run a rescan to detect missing files
- Check if files were moved or deleted externally
- Restore from trash if recently deleted

**Corrupted Files:**
- Run integrity check to detect corruption
- Replace with a different version if available
- Restore from backup if available

**Slow Performance:**
- Check database size (Settings → Database)
- Run VACUUM to reclaim space
- Reduce number of versions per media
- Archive old storage to separate drive

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+F | Open search |
| Ctrl+N | New collection |
| Ctrl+E | Export collection |
| Ctrl+S | Save search |
| Ctrl+, | Open settings |
| Esc | Close detail panel |

---

## Getting Help

- **In-app Help**: Click the "?" icon in the top-right corner
- **Documentation**: Visit the docs folder for detailed guides
- **Logs**: Check application logs for error details
- **Support**: Report issues on the project repository

