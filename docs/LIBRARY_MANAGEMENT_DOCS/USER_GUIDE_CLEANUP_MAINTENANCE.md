# User Guide: Cleanup and Maintenance

## Overview

The cleanup and maintenance features help you manage disk space, detect issues, and keep your library in optimal condition. This guide covers rescan operations, integrity checking, soft-delete and trash management, and cleanup operations.

## Table of Contents

1. [Rescan Operations](#rescan-operations)
2. [Integrity Checking](#integrity-checking)
3. [Soft Delete and Trash](#soft-delete-and-trash)
4. [Cleanup Operations](#cleanup-operations)
5. [Storage Analytics](#storage-analytics)
6. [Database Maintenance](#database-maintenance)
7. [Troubleshooting](#troubleshooting)

---

## Rescan Operations

### What is a Rescan?

A rescan reconciles your database with the actual filesystem state. It detects:

- **New Files**: Files on disk not in the database
- **Missing Files**: Files in database but not on disk
- **Status Changes**: Files that have been moved or deleted externally

### Running a Rescan

**Manual Rescan:**

1. Click "Maintenance" → "Rescan Library"
2. Select scope:
   - **All Roots**: Scan all library roots
   - **Specific Root**: Scan one library root
3. Click "Start Rescan"
4. Monitor progress:
   - Current directory being scanned
   - Files processed
   - New files found
   - Missing files detected

**Scheduled Rescan:**

1. Click "Settings" → "Maintenance"
2. Enable "Scheduled Rescan"
3. Set frequency (daily, weekly, monthly)
4. Set time to run (e.g., 2:00 AM)
5. Click "Save"

### Rescan Results

After rescan completes, review the report:

**New Files:**
- Files found on disk but not in database
- Option to import them
- Shows file size and location

**Missing Files:**
- Files in database but not on disk
- Marked with "missing" status
- Can be restored from trash if recently deleted

**Status Changes:**
- Files that were moved or renamed
- Updated in database
- May need manual review

### Handling New Files

For each new file found:

1. Review file details (name, size, location)
2. Click "Import" to add to library
3. Or click "Ignore" to skip
4. Metadata will be extracted during import

### Handling Missing Files

For each missing file:

1. Check if file was deleted intentionally
2. If deleted recently, restore from trash
3. If moved, update the path manually
4. Or mark as permanently missing

---

## Integrity Checking

### What is Integrity Checking?

Integrity checking verifies that:

- Files exist at recorded paths
- File sizes match database records
- File checksums match (if stored)
- No bit rot or corruption has occurred

### Running Integrity Checks

**Manual Check:**

1. Click "Maintenance" → "Check Integrity"
2. Select scope:
   - **All Files**: Check entire library
   - **Specific MediaItem**: Check one item
   - **Specific Version**: Check one file
3. Click "Start Check"
4. Monitor progress:
   - Files checked
   - Errors found
   - Estimated time remaining

**Scheduled Check:**

1. Click "Settings" → "Maintenance"
2. Enable "Scheduled Integrity Check"
3. Set frequency (weekly, monthly)
4. Set time to run
5. Set sampling rate (e.g., check 10% of library per run)
6. Click "Save"

### Integrity Check Results

**Passed:**
- File exists at recorded path
- File size matches
- Checksum matches (if available)
- No action needed

**Failed - File Missing:**
- File not found at recorded path
- Status updated to "missing"
- Consider restoring from backup

**Failed - Size Mismatch:**
- File size doesn't match database
- Possible file corruption
- Consider replacing with different version

**Failed - Checksum Mismatch:**
- File content has changed
- Possible bit rot or corruption
- Status marked as "corrupted"
- Consider replacing with different version

### Handling Corrupted Files

When corruption is detected:

1. **Alert**: Desktop notification is sent
2. **Status**: File marked as "corrupted"
3. **Options**:
   - Replace with different version (if available)
   - Restore from backup
   - Delete and re-import
   - Keep and monitor

**To replace corrupted file:**
1. Open media detail panel
2. Find corrupted version
3. Click "Replace"
4. Select replacement version
5. Corrupted file is moved to trash

---

## Soft Delete and Trash

### Understanding Soft Delete

Soft delete is a safe way to remove files:

- **Non-destructive**: Files are moved to trash, not permanently deleted
- **Recoverable**: Files can be restored within grace period
- **Audited**: All deletions are logged
- **Graceful**: Prevents accidental data loss

### Deleting Files

**Delete Single Version:**

1. Open media detail panel
2. Find the version to delete
3. Click "Delete"
4. Confirm deletion
5. File is moved to trash

**Delete Multiple Versions:**

1. Select multiple versions (checkbox)
2. Click "Delete Selected"
3. Confirm deletion
4. Files are moved to trash

**Delete Entire MediaItem:**

1. Open media detail panel
2. Click "Delete MediaItem"
3. Confirm deletion
4. All versions are moved to trash

### Trash Management

**Viewing Trashed Files:**

1. Click "Maintenance" → "Trash"
2. See all trashed files:
   - Original path
   - Deletion date
   - Days until permanent deletion
   - File size

**Restoring from Trash:**

1. Click "Maintenance" → "Trash"
2. Find the file to restore
3. Click "Restore"
4. File is moved back to original location
5. Status is updated to "present"

**Grace Period:**

- Default: 30 days
- Configurable in Settings → Maintenance
- After grace period, files are permanently deleted
- Countdown shown in trash view

### Trash Cleanup

**Automatic Cleanup:**

1. Click "Settings" → "Maintenance"
2. Enable "Auto-purge Trash"
3. Set grace period (days)
4. Click "Save"
5. Files older than grace period are automatically deleted

**Manual Cleanup:**

1. Click "Maintenance" → "Trash"
2. Click "Purge Trash"
3. Confirm permanent deletion
4. Files older than grace period are deleted
5. Review purge report

---

## Cleanup Operations

### Understanding Cleanup

Cleanup identifies and removes unnecessary files to free disk space:

**Cleanup Candidates:**
- Exact duplicates (same file hash)
- Lower-quality variants
- Trashed files past grace period
- Missing file records

### Running Cleanup

**View Cleanup Candidates:**

1. Click "Cleanup" in the sidebar
2. See list of candidates:
   - File path and size
   - Reason for candidacy
   - Quality score (if variant)
   - Estimated space savings

**Filter Candidates:**

1. Use filters to narrow list:
   - **Type**: Duplicates, variants, trashed, missing
   - **Size**: Minimum file size
   - **Quality**: Minimum quality score
   - **Age**: Files older than X days

**Select Files to Delete:**

1. Check boxes next to files to delete
2. Or use "Select All" to select all candidates
3. View total space that will be freed
4. Review selection carefully

**Execute Cleanup:**

1. Click "Execute Cleanup"
2. Review confirmation:
   - Number of files to delete
   - Total space to be freed
   - Estimated time
3. Click "Confirm"
4. Cleanup begins

### Cleanup Progress

During cleanup:

- Progress bar shows files processed
- Current file being deleted
- Errors are logged but don't stop process
- Can pause or cancel cleanup

### Cleanup Report

After cleanup completes:

- **Files Deleted**: Number of files removed
- **Space Freed**: Total bytes freed
- **Errors**: Any files that failed to delete
- **Time Taken**: Duration of cleanup

**Errors:**
- Logged for investigation
- File may still exist if error occurred
- Can retry cleanup for failed files

### Cleanup Safety Features

**Protected Files:**
- Preferred versions are never deleted
- User-edited metadata is preserved
- Audit log entries are created
- Soft-delete allows recovery

**Error Handling:**
- Errors don't stop the process
- Failed files are logged
- Can retry cleanup later
- No data loss if errors occur

---

## Storage Analytics

### Viewing Storage Statistics

**Storage Overview:**

1. Click "Storage Analytics" in the sidebar
2. View statistics:
   - **Total Size**: Combined size of all files
   - **MediaItems**: Number of unique media items
   - **FileVersions**: Total number of files
   - **Average Versions**: Average versions per media

**Breakdown by Category:**

- **Resolution**: 4K, 1080p, 720p, SD
- **Quality Label**: BluRay, WEB-DL, HDTV, DVD
- **Status**: Present, Missing, Trashed, Corrupted

### Duplicate Analysis

**Exact Duplicates:**

- Files with identical content (same hash)
- Wasted space: (count - 1) × file size
- Candidates for cleanup

**Variants:**

- Same MediaItem, different versions
- Different quality or resolution
- May want to keep best version only

**Duplicate Space:**

- Total space used by duplicates
- Space that could be freed
- Shown as percentage of total

### Largest Media Items

View your largest media items:

- **By Total Size**: All versions combined
- **By Single File**: Largest individual file
- **By Version Count**: Most versions

**Use cases:**
- Identify space hogs
- Find candidates for cleanup
- Understand storage distribution

### Storage Trends

Track storage usage over time:

- **Daily**: Storage used each day
- **Weekly**: Average per week
- **Monthly**: Average per month
- **Growth Rate**: How fast storage is growing

---

## Database Maintenance

### Database Optimization

**VACUUM Operation:**

1. Click "Settings" → "Database"
2. Click "Optimize Database"
3. Confirm optimization
4. Database is compacted
5. Unused space is reclaimed

**When to VACUUM:**
- After large cleanup operations
- After deleting many files
- Periodically (monthly)
- When database file is large

**Impact:**
- Reclaims disk space
- Improves query performance
- Takes a few minutes
- Database is locked during operation

### Index Optimization

**Rebuild Indexes:**

1. Click "Settings" → "Database"
2. Click "Rebuild Indexes"
3. Confirm rebuild
4. Indexes are rebuilt
5. Query performance may improve

**When to rebuild:**
- After large data changes
- If queries are slow
- Periodically (quarterly)

### Orphaned Record Cleanup

**Identify Orphaned Records:**

1. Click "Settings" → "Database"
2. Click "Check for Orphaned Records"
3. Review report:
   - Tags not used by any media
   - Collections with no items
   - Other orphaned data

**Remove Orphaned Records:**

1. Click "Clean Up Orphaned Records"
2. Confirm cleanup
3. Orphaned records are deleted
4. Review cleanup report

### Database Corruption Recovery

**Check Database Integrity:**

1. Click "Settings" → "Database"
2. Click "Check Integrity"
3. Review results:
   - Integrity check passed/failed
   - Errors found (if any)

**Recover from Corruption:**

1. If corruption detected:
   - Automatic recovery is attempted
   - If unsuccessful, restore from backup
   - Contact support if issues persist

---

## Troubleshooting

### Rescan Issues

**Rescan Hangs:**
- Check disk I/O performance
- Try canceling and restarting
- Check for very large directories
- Restart application

**New Files Not Detected:**
- Ensure files are in library root
- Check file permissions
- Run rescan again
- Check application logs

**Missing Files Not Detected:**
- Files may have been moved, not deleted
- Check trash directory
- Run integrity check
- Check application logs

### Integrity Check Issues

**Check Hangs:**
- Large files take longer to check
- Check disk I/O performance
- Try checking specific files instead
- Restart application

**False Positives:**
- File size changed (re-encoded)
- Checksum not available
- File was modified externally
- Review file manually

### Cleanup Issues

**Cleanup Hangs:**
- Large files take longer to delete
- Check disk I/O performance
- Try canceling and restarting
- Restart application

**Files Not Deleted:**
- File may be in use
- Permission issues
- Disk full
- Check application logs

**Wrong Files Deleted:**
- Review cleanup report
- Restore from trash if within grace period
- Restore from backup if available
- Check selection carefully next time

### Database Issues

**Database Locked:**
- Another operation is in progress
- Wait for operation to complete
- Restart application if stuck
- Check application logs

**Database Corrupted:**
- Run integrity check
- Attempt automatic recovery
- Restore from backup
- Contact support if issues persist

**Slow Queries:**
- Run VACUUM to optimize
- Rebuild indexes
- Check available disk space
- Monitor system resources

---

## Best Practices

### Regular Maintenance

- **Weekly**: Run rescan to detect changes
- **Monthly**: Run integrity check on sample of files
- **Monthly**: Run cleanup to remove duplicates
- **Quarterly**: Optimize database (VACUUM)

### Disk Space Management

- **Monitor Usage**: Check storage analytics regularly
- **Set Alerts**: Configure disk space threshold
- **Plan Cleanup**: Schedule cleanup during off-hours
- **Archive Old**: Move old media to archive storage

### Data Safety

- **Backup Important**: Keep copies of critical media
- **Use Preferred**: Mark your best quality versions
- **Monitor Corruption**: Check integrity regularly
- **Review Logs**: Check audit logs for unexpected changes

### Performance

- **Batch Operations**: Group operations together
- **Off-hours**: Run intensive operations at night
- **Monitor Resources**: Check CPU and disk usage
- **Optimize Database**: Regular VACUUM and index rebuild

---

## Advanced Topics

### Batch Cleanup Configuration

Configure cleanup behavior:

1. Click "Settings" → "Cleanup"
2. Set options:
   - **Batch Size**: Files to delete per batch
   - **Batch Delay**: Delay between batches
   - **Error Handling**: Continue or stop on error
3. Click "Save"

**Recommendations:**
- Batch size 10-50 for HDD
- Batch size 50-100 for SSD
- Delay 100-500ms for HDD
- Delay 10-50ms for SSD

### Rescan Batch Configuration

Configure rescan behavior:

1. Click "Settings" → "Rescan"
2. Set options:
   - **Batch Size**: Files to process per batch
   - **Batch Delay**: Delay between batches
   - **Parallel Threads**: Number of parallel scans
3. Click "Save"

**Recommendations:**
- Batch size 100-500 for HDD
- Batch size 500-1000 for SSD
- Delay 50-200ms for HDD
- Delay 5-20ms for SSD

### Integrity Check Sampling

For large libraries, check a sample:

1. Click "Settings" → "Integrity"
2. Set "Sampling Rate" (e.g., 10%)
3. Set "Sampling Strategy":
   - **Random**: Random sample
   - **Oldest**: Oldest files first
   - **Largest**: Largest files first
4. Click "Save"

**Recommendations:**
- 10% for large libraries (10,000+ files)
- 25% for medium libraries (1,000-10,000 files)
- 100% for small libraries (<1,000 files)

---

## Getting Help

- **Maintenance Logs**: Check logs for detailed information
- **Audit Log**: Review audit log for operation history
- **Storage Analytics**: Use to understand storage usage
- **Support**: Report issues with specific operations

