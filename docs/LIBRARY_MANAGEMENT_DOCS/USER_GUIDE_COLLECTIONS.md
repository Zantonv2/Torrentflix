# User Guide: Collections and Smart Collections

## Overview

Collections let you organize media items into custom groupings. TorrentFlix supports two types of collections:

- **Manual Collections**: Manually add items to static collections
- **Smart Collections**: Dynamically updated collections based on filter criteria

This guide covers creating, managing, and using both types of collections.

## Table of Contents

1. [Understanding Collections](#understanding-collections)
2. [Manual Collections](#manual-collections)
3. [Smart Collections](#smart-collections)
4. [Managing Collections](#managing-collections)
5. [Exporting Collections](#exporting-collections)
6. [Collection Examples](#collection-examples)
7. [Best Practices](#best-practices)

---

## Understanding Collections

### What are Collections?

Collections are custom groupings of media items. They help you:

- **Organize**: Group related content together
- **Discover**: Find media by theme or category
- **Share**: Export collections for sharing
- **Automate**: Use smart collections for dynamic grouping

### Collection Types

**Manual Collections:**
- You explicitly add items
- Static membership (doesn't change automatically)
- Good for curated lists
- Examples: "Favorites", "To-watch", "Movie Series"

**Smart Collections:**
- Items are added automatically based on criteria
- Dynamic membership (updates automatically)
- Good for categories
- Examples: "4K Movies", "Unwatched", "Recent Additions"

### Collection Features

**Browsing:**
- View all items in a collection
- Apply filters and sorting
- Search within collection

**Management:**
- Add/remove items
- Edit collection details
- Delete collections
- Export collections

**Integration:**
- Use in import profiles
- Reference in cleanup operations
- Include in backups

---

## Manual Collections

### Creating a Manual Collection

**Step 1: Create Collection**

1. Click "Collections" in the sidebar
2. Click "New Collection"
3. Enter collection details:
   - **Name**: Collection name (required)
   - **Description**: Optional description
   - **Type**: Select "Manual Collection"
4. Click "Create"

**Step 2: Add Items**

1. Open the collection
2. Click "Add Items"
3. Select items to add:
   - Search for items
   - Filter by criteria
   - Select multiple items (checkbox)
4. Click "Add Selected"

**Step 3: Manage Items**

1. View items in collection
2. Remove items:
   - Click "Remove" next to item
   - Or select multiple and click "Remove Selected"
3. Reorder items (if supported):
   - Drag items to reorder
   - Or use up/down arrows

### Adding Items to Collections

**From Library View:**

1. Select one or more items
2. Click "Add to Collection"
3. Choose collection
4. Click "Add"

**From Media Detail:**

1. Open media detail panel
2. Scroll to "Collections" section
3. Click "Add to Collection"
4. Select collection
5. Click "Add"

**Batch Add:**

1. Select multiple items in library
2. Click "Add to Collection"
3. Choose collection
4. Click "Add All"

### Removing Items from Collections

**From Collection View:**

1. Open collection
2. Find item to remove
3. Click "Remove"
4. Confirm removal

**From Media Detail:**

1. Open media detail panel
2. Find collection in "Collections" section
3. Click "Remove"
4. Confirm removal

**Batch Remove:**

1. Open collection
2. Select multiple items
3. Click "Remove Selected"
4. Confirm removal

### Editing Collection Details

**To edit collection:**

1. Open collection
2. Click "Edit"
3. Modify:
   - Name
   - Description
4. Click "Save"

**To delete collection:**

1. Open collection
2. Click "Delete Collection"
3. Confirm deletion
4. Items are not deleted, only collection is removed

---

## Smart Collections

### Understanding Smart Collections

Smart collections automatically include items matching specified criteria:

- **Dynamic**: Items are added/removed automatically
- **Real-time**: Updates happen immediately
- **Criteria-based**: Based on filters you define
- **No Manual Maintenance**: No need to manually add items

### Creating a Smart Collection

**Step 1: Create Collection**

1. Click "Collections" in the sidebar
2. Click "New Smart Collection"
3. Enter collection details:
   - **Name**: Collection name (required)
   - **Description**: Optional description
   - **Type**: Select "Smart Collection"
4. Click "Next"

**Step 2: Define Criteria**

1. Set filter criteria:
   - **Title Pattern**: Match by title (e.g., "Marvel")
   - **Year Range**: Match by year (e.g., 2020-2023)
   - **Resolution**: Match by resolution (4K, 1080p, 720p, SD)
   - **Quality Label**: Match by quality (BluRay, WEB-DL, etc.)
   - **Tags**: Match by tags (any or all)
   - **Rating**: Minimum user rating (0.0-10.0)
   - **Watch Status**: Match by watch status
   - **Version Count**: Match by number of versions
2. Click "Next"

**Step 3: Review and Create**

1. Review criteria
2. See preview of matching items
3. Click "Create"

### Smart Collection Criteria

**Title Pattern:**
- Partial match (case-insensitive)
- Supports wildcards: `*` (any characters)
- Examples:
  - `Marvel` - matches "Marvel Cinematic Universe", "Marvel Studios"
  - `*Avengers*` - matches "Avengers", "Avengers: Endgame"
  - `The *` - matches "The Matrix", "The Dark Knight"

**Year Range:**
- Inclusive range (e.g., 2020-2023)
- Single year (e.g., 2023)
- Open-ended (e.g., 2020+)

**Resolution:**
- 4K (2160p and above)
- 1080p (1920x1080)
- 720p (1280x720)
- SD (below 720p)

**Quality Label:**
- BluRay
- WEB-DL
- HDTV
- DVD
- Other

**Tags:**
- Match any tag (OR logic)
- Match all tags (AND logic)
- Multiple tags can be selected

**Rating:**
- Minimum user rating (0.0-10.0)
- Only items with rating >= threshold

**Watch Status:**
- Unwatched
- In Progress
- Watched

**Version Count:**
- Minimum number of versions
- Maximum number of versions
- Exact number of versions

### Automatic Updates

Smart collections update automatically when:

- **New Item Added**: If it matches criteria, it's added to collection
- **Item Modified**: If criteria no longer match, it's removed
- **Metadata Changed**: If changes affect criteria match
- **Tags Added/Removed**: If tags are part of criteria
- **Rating Changed**: If rating is part of criteria

**Example:**
- Smart collection "Unwatched 1080p" with criteria: watch status = unwatched, resolution = 1080p
- When you mark an item as watched, it's automatically removed
- When you import a new 1080p unwatched item, it's automatically added

### Editing Smart Collections

**To edit criteria:**

1. Open smart collection
2. Click "Edit Criteria"
3. Modify filter criteria
4. Click "Save"
5. Collection updates automatically

**To edit details:**

1. Open smart collection
2. Click "Edit"
3. Modify name or description
4. Click "Save"

**To delete collection:**

1. Open smart collection
2. Click "Delete Collection"
3. Confirm deletion
4. Items are not deleted, only collection is removed

---

## Managing Collections

### Viewing Collections

**Collections List:**

1. Click "Collections" in the sidebar
2. See all collections:
   - Manual collections
   - Smart collections
   - Number of items in each
   - Last modified date

**Collection Details:**

1. Click on a collection
2. View:
   - Collection name and description
   - Items in collection
   - Collection type (manual or smart)
   - Criteria (if smart collection)

### Browsing Collection Items

**View Items:**

1. Open collection
2. See items in grid or list view
3. Same browsing features as library:
   - Filter and sort
   - Search within collection
   - View media details

**Apply Filters:**

1. Open collection
2. Click "Filters"
3. Apply additional filters
4. Filters are combined with collection criteria

**Sort Items:**

1. Open collection
2. Click "Sort"
3. Choose sort order:
   - Title (A-Z)
   - Year (newest first)
   - Added date (newest first)
   - Size (largest first)
   - Rating (highest first)

### Searching Within Collections

**Quick Search:**

1. Open collection
2. Use search bar
3. Search is limited to collection items

**Advanced Search:**

1. Open collection
2. Click "Filters"
3. Apply filters
4. Results are limited to collection items

### Collection Statistics

View statistics for a collection:

1. Open collection
2. Click "Statistics"
3. View:
   - Number of items
   - Total size
   - Average rating
   - Breakdown by resolution, quality, status

---

## Exporting Collections

### Export Formats

Collections can be exported in multiple formats:

**JSON Format:**
- Complete metadata
- Preserves all information
- Good for backup or import
- Human-readable

**CSV Format:**
- Spreadsheet-compatible
- Simplified format
- Good for sharing
- Easy to edit

### Exporting a Collection

**Step 1: Open Collection**

1. Click "Collections" in the sidebar
2. Open the collection to export

**Step 2: Export**

1. Click "Export"
2. Choose format:
   - JSON
   - CSV
3. Select what to include:
   - File paths
   - Metadata (title, year, genres, etc.)
   - Ratings and tags
   - Watch status
   - Custom notes
4. Click "Export"

**Step 3: Save File**

1. Choose save location
2. Enter filename
3. Click "Save"

### Export Contents

**JSON Export:**

```json
{
  "collection": {
    "name": "Favorites",
    "description": "My favorite movies",
    "type": "manual",
    "created_at": "2023-01-15T10:30:00Z",
    "items": [
      {
        "title": "The Matrix",
        "year": 1999,
        "genres": ["Action", "Sci-Fi"],
        "rating": 9.5,
        "versions": [
          {
            "path": "/library/The Matrix (1999)/The Matrix (1999) - 1080p.mkv",
            "size": 2500000000,
            "resolution": "1920x1080"
          }
        ]
      }
    ]
  }
}
```

**CSV Export:**

```
Title,Year,Genres,Rating,Path,Size,Resolution
The Matrix,1999,"Action, Sci-Fi",9.5,/library/The Matrix (1999)/The Matrix (1999) - 1080p.mkv,2500000000,1920x1080
```

### Importing Collections

**To import a collection:**

1. Click "Collections" in the sidebar
2. Click "Import Collection"
3. Select JSON or CSV file
4. Choose import options:
   - **Merge**: Add items to existing collection
   - **Replace**: Replace existing collection
   - **New**: Create new collection
5. Click "Import"

---

## Collection Examples

### Movie Series

**Manual Collection: Marvel Cinematic Universe**

1. Create manual collection "MCU"
2. Add all Marvel movies in order
3. Use for watching series in sequence

**Smart Collection: MCU Movies**

1. Create smart collection
2. Criteria: Title pattern = "*Marvel*" OR "*Avengers*"
3. Automatically includes all MCU movies

### Quality-Based Collections

**Smart Collection: 4K Movies**

- Criteria: Resolution = 4K
- Automatically includes all 4K content
- Updates when new 4K files are added

**Smart Collection: BluRay Collection**

- Criteria: Quality label = BluRay
- Automatically includes all BluRay files
- Good for tracking high-quality versions

### Status-Based Collections

**Smart Collection: Unwatched**

- Criteria: Watch status = Unwatched
- Automatically includes all unwatched items
- Removes items when watched

**Smart Collection: In Progress**

- Criteria: Watch status = In Progress
- Automatically includes items being watched
- Good for resuming viewing

### Time-Based Collections

**Smart Collection: Recent Additions**

- Criteria: Added date within last 30 days
- Automatically includes newly imported items
- Updates daily

**Smart Collection: 2023 Releases**

- Criteria: Year = 2023
- Automatically includes all 2023 movies
- Good for tracking recent releases

### Rating-Based Collections

**Smart Collection: Favorites**

- Criteria: User rating >= 8.0
- Automatically includes highly-rated items
- Updates when ratings change

**Smart Collection: To-Watch**

- Criteria: Watch status = Unwatched AND Rating >= 7.0
- Automatically includes unwatched highly-rated items
- Good for finding what to watch next

### Tag-Based Collections

**Smart Collection: Action Movies**

- Criteria: Tags = "Action"
- Automatically includes all action-tagged items
- Updates when tags are added/removed

**Smart Collection: Family Friendly**

- Criteria: Tags = "Family" AND Tags = "Friendly"
- Automatically includes items with both tags
- Good for filtering by content type

---

## Best Practices

### Organization

- **Use Consistent Naming**: Clear, descriptive names
- **Add Descriptions**: Explain collection purpose
- **Organize Hierarchically**: Use naming conventions (e.g., "Genre: Action")
- **Regular Review**: Remove unused collections

### Manual Collections

- **Curate Carefully**: Add only items that fit theme
- **Maintain Order**: Keep items in logical order
- **Document Purpose**: Use description field
- **Update Regularly**: Add new items as they arrive

### Smart Collections

- **Use Clear Criteria**: Make criteria obvious from name
- **Test Criteria**: Verify collection includes expected items
- **Avoid Overlap**: Minimize duplicate items across collections
- **Monitor Updates**: Verify automatic updates work correctly

### Performance

- **Limit Collections**: Too many collections can slow browsing
- **Simplify Criteria**: Complex criteria may be slow
- **Archive Old**: Remove unused collections
- **Batch Operations**: Add multiple items at once

### Sharing

- **Export Regularly**: Backup important collections
- **Use JSON Format**: Preserves all information
- **Document Collections**: Include notes about purpose
- **Version Control**: Keep track of collection versions

---

## Advanced Topics

### Complex Criteria

Combine multiple criteria for powerful collections:

**Example: "Best Recent 4K"**
- Resolution = 4K
- Year >= 2020
- User rating >= 8.0
- Quality label = BluRay

**Example: "Unwatched Sci-Fi"**
- Watch status = Unwatched
- Tags = "Sci-Fi"
- Rating >= 7.0

### Collection Workflows

**Workflow: Watch Queue**

1. Create smart collection "To-Watch"
   - Criteria: Watch status = Unwatched, Rating >= 7.0
2. Sort by rating (highest first)
3. Watch items in order
4. Mark as watched when done
5. Items automatically removed from collection

**Workflow: Quality Upgrade**

1. Create smart collection "Lower Quality"
   - Criteria: Resolution < 1080p
2. Identify items to upgrade
3. Import higher-quality versions
4. Delete lower-quality versions
5. Items automatically removed from collection

### Batch Operations on Collections

**Add Multiple Items:**

1. Select items in library
2. Click "Add to Collection"
3. Choose collection
4. All selected items added at once

**Remove Multiple Items:**

1. Open collection
2. Select items
3. Click "Remove Selected"
4. All selected items removed at once

**Apply Tags to Collection:**

1. Open collection
2. Select all items
3. Click "Add Tags"
4. Select tags
5. Tags applied to all items

---

## Getting Help

- **Collection Tips**: Hover over fields for help text
- **Examples**: See collection examples above
- **Criteria Help**: Click "?" next to criteria fields
- **Support**: Report issues with collections

