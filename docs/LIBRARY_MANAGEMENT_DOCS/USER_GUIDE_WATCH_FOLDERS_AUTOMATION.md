# User Guide: Watch Folders and Automation

## Overview

Watch folders enable automatic importing of media files as they appear in designated directories. Combined with import profiles, they provide powerful automation for organizing your library with minimal manual intervention.

## Table of Contents

1. [Understanding Watch Folders](#understanding-watch-folders)
2. [Setting Up Watch Folders](#setting-up-watch-folders)
3. [Import Profiles](#import-profiles)
4. [Automation Rules](#automation-rules)
5. [Monitoring and Troubleshooting](#monitoring-and-troubleshooting)
6. [Advanced Automation](#advanced-automation)
7. [Best Practices](#best-practices)

---

## Understanding Watch Folders

### What are Watch Folders?

Watch folders are directories that TorrentFlix monitors for new files. When files appear and finish writing, they're automatically imported into your library.

**Key Features:**
- **Automatic Detection**: New files are detected immediately
- **Write Completion**: Waits for file to finish writing
- **Recursive Monitoring**: Can monitor subdirectories
- **Profile Support**: Apply import rules automatically
- **Error Handling**: Failed imports are logged and preserved

### How Watch Folders Work

```
File appears in watch folder
    ↓
System detects file
    ↓
Waits for write completion (file stops changing)
    ↓
Applies import profile (if configured)
    ↓
Stages file (copy to staging directory)
    ↓
Probes metadata
    ↓
Commits to library
    ↓
File is now in your library
```

### Write Completion Detection

The system detects when a file has finished writing:

**Methods:**
- **Timeout-based**: File hasn't changed for 5 seconds
- **Size-based**: File size stabilizes
- **Inotify Events**: Linux filesystem events

**Typical Scenarios:**
- Download completes → file is imported
- Torrent finishes → file is imported
- Manual file copy → file is imported after copy completes
- Streaming download → file is imported when complete

---

## Setting Up Watch Folders

### Creating a Watch Folder

**Step 1: Open Watch Folder Settings**

1. Click "Settings" in the main menu
2. Select "Watch Folders"
3. Click "Add Watch Folder"

**Step 2: Configure Watch Folder**

1. **Select Directory**:
   - Click "Browse"
   - Navigate to directory
   - Click "Select"

2. **Configure Options**:
   - **Name**: Descriptive name (e.g., "Downloads")
   - **Recursive**: Monitor subdirectories (yes/no)
   - **Initial Scan**: Import existing files (yes/no)
   - **Import Profile**: Select profile to apply (optional)
   - **Target Library Root**: Where to store files (optional)

3. **Review Settings**:
   - Verify directory path
   - Check options are correct
   - Review profile settings

**Step 3: Save Watch Folder**

1. Click "Save"
2. Watch folder is created and monitoring begins
3. See confirmation message

### Watch Folder Options

**Recursive Monitoring:**
- **Enabled**: Monitor all subdirectories
- **Disabled**: Only monitor top-level directory
- Use for: Downloads folder with subfolders

**Initial Scan:**
- **Enabled**: Import existing files in directory
- **Disabled**: Only import new files going forward
- Use for: Importing existing downloads

**Import Profile:**
- **None**: Import without profile rules
- **Selected Profile**: Apply profile rules
- Use for: Automatic tagging, quality filtering

**Target Library Root:**
- **Auto-detect**: Use default library root
- **Specific Root**: Use selected library root
- Use for: Organizing by storage device

### Example Watch Folder Configurations

**Configuration 1: Downloads Folder**

- Directory: `/home/user/Downloads`
- Recursive: Yes
- Initial Scan: No
- Import Profile: None
- Target Root: Main HDD

**Configuration 2: Torrent Downloads**

- Directory: `/home/user/Downloads/Torrents`
- Recursive: Yes
- Initial Scan: Yes
- Import Profile: "Torrent Profile"
- Target Root: Main HDD

**Configuration 3: Archive Imports**

- Directory: `/mnt/external/Archive`
- Recursive: Yes
- Initial Scan: Yes
- Import Profile: "Archive Profile"
- Target Root: Archive Storage

---

## Import Profiles

### Understanding Import Profiles

Import profiles are automation rules that apply to imported files:

- **Pattern Matching**: Match files by name pattern
- **Quality Filtering**: Enforce quality requirements
- **Auto-Tagging**: Automatically add tags
- **Collection Assignment**: Add to collections
- **Priority-Based**: Higher priority profiles are checked first

### Creating an Import Profile

**Step 1: Open Import Profiles**

1. Click "Settings" in the main menu
2. Select "Import Profiles"
3. Click "New Profile"

**Step 2: Basic Information**

1. **Name**: Profile name (e.g., "4K Movies")
2. **Description**: Optional description
3. **Priority**: Higher number = higher priority
4. **Enabled**: Toggle to enable/disable profile

**Step 3: File Matching**

1. **File Pattern**: Pattern to match files
   - Glob pattern: `*.mkv`, `*4K*`, `*BluRay*`
   - Regex pattern: `.*4K.*`, `.*\(2023\).*`
2. **Test Pattern**: Enter test filename to verify

**Step 4: Quality Filters**

1. **Minimum Resolution**: Reject files below this (optional)
   - Options: 720p, 1080p, 1440p, 2160p (4K)
2. **Allowed Codecs**: Only accept specific codecs (optional)
   - Options: H.264, H.265, VP9, AV1, etc.
3. **Minimum Bitrate**: Reject low-bitrate files (optional)

**Step 5: Automation Rules**

1. **Auto-Tags**: Tags to apply automatically
   - Select existing tags or create new ones
   - Multiple tags can be selected
2. **Target Collection**: Collection to add to (optional)
   - Select from existing collections
   - Items are automatically added
3. **Target Library Root**: Where to store files (optional)
   - Select from configured library roots
   - Overrides watch folder setting

**Step 6: Save Profile**

1. Review all settings
2. Click "Save"
3. Profile is created and ready to use

### Profile Matching Examples

**Profile: 4K Movies**

- Pattern: `*4K*` or `*2160p*`
- Min Resolution: 2160p
- Allowed Codecs: H.265, VP9
- Auto-Tags: "4K", "Premium"
- Target Collection: "4K Collection"

**Profile: BluRay**

- Pattern: `*BluRay*` or `*BR*`
- Min Resolution: 1080p
- Allowed Codecs: H.264, H.265
- Auto-Tags: "BluRay", "High-Quality"
- Target Collection: "BluRay Collection"

**Profile: Archive**

- Pattern: `*` (matches all)
- Min Resolution: None
- Allowed Codecs: Any
- Auto-Tags: "Archive"
- Target Library Root: "Archive Storage"

### Profile Priority

When multiple profiles match a file:

1. **Priority Order**: Profiles are checked in priority order
2. **First Match**: First matching profile is used
3. **No Match**: File is imported without profile rules

**Example Priority:**
1. Priority 100: "4K Movies" (most specific)
2. Priority 50: "BluRay" (medium specific)
3. Priority 10: "Default" (least specific)

### Editing and Deleting Profiles

**To Edit Profile:**

1. Click "Settings" → "Import Profiles"
2. Find profile in list
3. Click "Edit"
4. Modify settings
5. Click "Save"

**To Delete Profile:**

1. Click "Settings" → "Import Profiles"
2. Find profile in list
3. Click "Delete"
4. Confirm deletion

**To Disable Profile:**

1. Click "Settings" → "Import Profiles"
2. Find profile in list
3. Toggle "Enabled" to off
4. Profile is not used for matching

---

## Automation Rules

### Auto-Tagging

Automatically add tags to imported files:

**How It Works:**
1. File matches import profile
2. Tags specified in profile are applied
3. Tags are added to MediaItem
4. Tags can be used for filtering and organization

**Example:**
- Profile "4K Movies" has auto-tags: "4K", "Premium"
- File matching profile is imported
- Tags "4K" and "Premium" are automatically added
- Can filter library by these tags

### Collection Assignment

Automatically add imported files to collections:

**How It Works:**
1. File matches import profile
2. Target collection is specified in profile
3. MediaItem is added to collection
4. Collection membership is automatic

**Example:**
- Profile "BluRay" has target collection "BluRay Collection"
- File matching profile is imported
- MediaItem is automatically added to "BluRay Collection"
- Collection is updated in real-time

### Quality Filtering

Enforce quality requirements for imported files:

**How It Works:**
1. File matches import profile
2. Quality filters are checked
3. If file doesn't meet requirements, import is rejected
4. File is moved to failed imports directory

**Example:**
- Profile "4K Only" requires minimum resolution 2160p
- File with 1080p resolution matches pattern
- Quality filter rejects file
- File is moved to failed imports directory

### Target Library Root

Specify where imported files are stored:

**How It Works:**
1. File matches import profile
2. Target library root is specified in profile
3. File is stored in specified root
4. Overrides watch folder default

**Example:**
- Profile "Archive" has target root "Archive Storage"
- File matches pattern
- File is stored in "Archive Storage" instead of default root
- Useful for organizing by storage device

---

## Monitoring and Troubleshooting

### Viewing Watch Folder Status

**Watch Folder List:**

1. Click "Settings" → "Watch Folders"
2. See all watch folders:
   - Directory path
   - Status (active/inactive)
   - Last scan time
   - Number of files imported

**Watch Folder Details:**

1. Click on watch folder
2. View:
   - Configuration settings
   - Recent imports
   - Failed imports
   - Statistics

### Monitoring Imports

**Recent Imports:**

1. Click "Maintenance" → "Recent Imports"
2. See recently imported files:
   - File name and size
   - Import time
   - Applied profile
   - Status (success/failed)

**Failed Imports:**

1. Click "Maintenance" → "Failed Imports"
2. See files that failed import:
   - File name and size
   - Error message
   - Reason for failure
   - Option to retry

### Common Issues

**Files Not Being Imported**

**Symptoms:**
- Files appear in watch folder but aren't imported
- No error messages

**Causes:**
- Watch folder is disabled
- File write hasn't completed
- File doesn't match any profile
- Disk space is full
- Permission issues

**Solutions:**
1. Check watch folder is enabled
2. Wait for file write to complete
3. Check file matches profile pattern
4. Free up disk space
5. Check file permissions

**Import Fails with Quality Filter Error**

**Symptoms:**
- File matches pattern but import fails
- Error message about quality requirements

**Causes:**
- File resolution is below minimum
- File codec is not allowed
- File bitrate is below minimum

**Solutions:**
1. Check profile quality filters
2. Verify file meets requirements
3. Adjust profile filters if needed
4. Use different profile without filters

**Profile Not Applied**

**Symptoms:**
- File imported but profile rules not applied
- Tags not added, collection not updated

**Causes:**
- Profile is disabled
- File doesn't match pattern
- Profile priority is lower than another
- Profile configuration is incorrect

**Solutions:**
1. Check profile is enabled
2. Verify file matches pattern
3. Check profile priority
4. Review profile configuration

**Watch Folder Stops Monitoring**

**Symptoms:**
- Watch folder was working but stopped
- New files aren't imported

**Causes:**
- Watch folder was disabled
- Directory was deleted or moved
- Permission issues
- Application crashed

**Solutions:**
1. Check watch folder is enabled
2. Verify directory still exists
3. Check directory permissions
4. Restart application

### Debugging Watch Folders

**Check Application Logs:**

1. Click "Settings" → "Logs"
2. Filter by "Watch Folder"
3. Review log entries for errors

**Test File Pattern:**

1. Click "Settings" → "Import Profiles"
2. Find profile
3. Click "Test Pattern"
4. Enter test filename
5. See if pattern matches

**Manual Import Test:**

1. Manually import a file from watch folder
2. Check if import succeeds
3. If manual import works, watch folder issue is likely timing-related

---

## Advanced Automation

### Complex Profile Patterns

**Glob Patterns:**

- `*.mkv` - All MKV files
- `*4K*` - Files containing "4K"
- `*BluRay*` - Files containing "BluRay"
- `[0-9]*.mp4` - MP4 files starting with number
- `*{2023,2024}*` - Files from 2023 or 2024

**Regex Patterns:**

- `.*4K.*` - Files containing "4K"
- `.*\(2023\).*` - Files containing "(2023)"
- `^[A-Z].*` - Files starting with uppercase
- `.*BluRay.*x265.*` - BluRay files with H.265

### Multi-Profile Workflows

**Workflow: Automatic Organization**

1. Create profiles:
   - "4K Movies": Pattern `*4K*`, target "4K Collection"
   - "BluRay": Pattern `*BluRay*`, target "BluRay Collection"
   - "Archive": Pattern `*`, target "Archive Storage"

2. Set priorities:
   - 4K Movies: 100
   - BluRay: 50
   - Archive: 10

3. Files are automatically organized:
   - 4K BluRay files go to "4K Collection"
   - Regular BluRay files go to "BluRay Collection"
   - Everything else goes to "Archive Storage"

**Workflow: Quality Enforcement**

1. Create profiles:
   - "Premium": Min resolution 1080p, allowed codecs H.265
   - "Standard": Min resolution 720p, allowed codecs H.264, H.265
   - "Archive": No quality requirements

2. Set priorities:
   - Premium: 100
   - Standard: 50
   - Archive: 10

3. Files are filtered by quality:
   - High-quality files go to "Premium" collection
   - Medium-quality files go to "Standard" collection
   - Low-quality files go to "Archive" collection

### Batch Profile Configuration

**To configure multiple profiles at once:**

1. Click "Settings" → "Import Profiles"
2. Click "Import Profiles"
3. Select JSON file with profile definitions
4. Profiles are imported and created

**Profile JSON Format:**

```json
{
  "profiles": [
    {
      "name": "4K Movies",
      "priority": 100,
      "pattern": "*4K*",
      "min_resolution": "2160p",
      "allowed_codecs": ["H.265", "VP9"],
      "auto_tags": ["4K", "Premium"],
      "target_collection": "4K Collection"
    },
    {
      "name": "BluRay",
      "priority": 50,
      "pattern": "*BluRay*",
      "min_resolution": "1080p",
      "allowed_codecs": ["H.264", "H.265"],
      "auto_tags": ["BluRay"],
      "target_collection": "BluRay Collection"
    }
  ]
}
```

---

## Best Practices

### Watch Folder Organization

- **Separate by Type**: Different folders for movies, TV, etc.
- **Use Descriptive Names**: Clear folder names
- **Organize by Source**: Separate folders for different sources
- **Regular Cleanup**: Remove imported files from watch folder

### Profile Design

- **Clear Naming**: Descriptive profile names
- **Specific Patterns**: Avoid overly broad patterns
- **Priority Management**: Set priorities carefully
- **Regular Review**: Update profiles as needed

### Automation Strategy

- **Start Simple**: Begin with basic profiles
- **Test Thoroughly**: Verify profiles work correctly
- **Monitor Results**: Check that automation works as expected
- **Iterate**: Adjust profiles based on results

### Performance

- **Limit Watch Folders**: Too many can impact performance
- **Optimize Patterns**: Complex patterns may be slow
- **Batch Operations**: Process files in batches
- **Monitor Resources**: Check CPU and disk usage

### Safety

- **Test Profiles**: Verify before using in production
- **Review Failed Imports**: Investigate failures
- **Backup Important**: Keep copies of critical files
- **Monitor Automation**: Ensure it's working correctly

---

## Troubleshooting Guide

### Watch Folder Not Detecting Files

**Check:**
1. Watch folder is enabled
2. Directory path is correct
3. Directory exists and is accessible
4. File write has completed (wait 5+ seconds)
5. File permissions allow reading

**Fix:**
1. Enable watch folder if disabled
2. Verify directory path
3. Create directory if missing
4. Wait for file write to complete
5. Check file permissions

### Profile Not Matching Files

**Check:**
1. Profile is enabled
2. File pattern is correct
3. Test pattern with sample filename
4. Profile priority is appropriate
5. No other profile is matching first

**Fix:**
1. Enable profile if disabled
2. Adjust file pattern
3. Use pattern tester to verify
4. Adjust priority if needed
5. Review other profiles

### Quality Filter Rejecting Files

**Check:**
1. File meets minimum resolution requirement
2. File codec is in allowed list
3. File bitrate meets minimum requirement
4. Profile filters are correct

**Fix:**
1. Verify file specifications
2. Adjust profile filters if needed
3. Use different profile without filters
4. Check file is valid media file

### Import Hangs or Freezes

**Check:**
1. Disk space is available
2. File is not corrupted
3. File write has completed
4. Application is responsive

**Fix:**
1. Free up disk space
2. Try different file
3. Wait for file write to complete
4. Restart application

---

## Getting Help

- **Profile Testing**: Use pattern tester to verify patterns
- **Logs**: Check application logs for detailed information
- **Failed Imports**: Review failed imports directory
- **Support**: Report issues with specific file types or patterns

