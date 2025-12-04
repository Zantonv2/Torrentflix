# 📋 UI REDESIGN - EXECUTIVE SUMMARY

## THE PLAN

**Full UI redesign** with lean component architecture.

### Current State
- 29 components (bloated)
- 7 tabs (confusing)
- 300+ lines in App.svelte (too big)
- 700+ lines in SettingsTab.svelte (way too big)
- No state management (props drilling)
- Inconsistent design

### New State
- 12 components (lean)
- 4 pages (clear)
- 50 lines in App.svelte (clean)
- Modular settings (organized)
- Svelte stores (clean state)
- Consistent design

---

## COMPONENT REDUCTION

| Category | Current | New | Reduction |
|----------|---------|-----|-----------|
| Core | 8 | 8 | 0% |
| Layout | 2 | 3 | +50% |
| Pages | 7 | 4 | -43% |
| Utilities | 12 | 0 | -100% |
| **TOTAL** | **29** | **12** | **-59%** |

---

## NEW STRUCTURE

```
App.svelte (50 lines)
├── MainLayout (80 lines)
│   ├── Header (60 lines)
│   ├── Sidebar (80 lines)
│   └── Content
│       ├── Discover (120 lines)
│       ├── Library (150 lines)
│       ├── Downloads (100 lines)
│       └── Settings (200 lines)
├── Core Components (8)
│   ├── Button
│   ├── Input
│   ├── Modal
│   ├── Card
│   ├── Badge
│   ├── Loading
│   ├── Toast
│   └── Tabs
└── Stores (5)
    ├── search
    ├── library
    ├── downloads
    ├── settings
    └── ui
```

---

## WHAT GETS DELETED

17 components consolidated into 12:

```
TopBar → Header
TabBar → Sidebar
MovieGrid → Discover
MovieCard → Discover (inline)
MovieTile → Discover (inline)
DetailPanel → Modal
DownloadsTab → Downloads
SettingsTab → Settings
LibraryBrowser → Library
MediaItemDetail → Modal
CollectionManager → Library
StorageAnalytics → Library
ManagementPanel → Library
JobsDashboard → Library
CleanupManager → Library
VersionManager → Library
SkeletonTile → Loading
LoadingSpinner → Loading
LoadingDots → Loading
SearchBar → Header
SecureInput → Input
SettingsSection → Card
SettingsNotification → Toast
```

---

## TIMELINE

| Week | Task | Hours |
|------|------|-------|
| 1 | Core components | 16 |
| 2 | Layout + Stores | 16 |
| 3 | Pages | 16 |
| 4 | Polish + Testing | 16 |
| **TOTAL** | | **64** |

---

## BACKEND CHANGES

**NONE**. All Tauri commands stay the same:
- search_movies()
- get_feed()
- start_download()
- get_active_downloads()
- pause_download()
- resume_download()
- delete_download()
- get_settings()
- save_settings()
- test_qbittorrent()
- pick_folder()
- set_user_rating()
- set_watch_status()
- set_custom_notes()
- set_preferred_version()
- query_library()
- search_library()
- get_media_item()
- get_file_versions()
- stage_file()
- commit_staged_file()
- get_staged_files()
- schedule_rescan()
- get_cleanup_candidates()
- execute_cleanup()
- get_storage_analytics()
- create_collection()
- add_to_collection()
- get_collection_items()
- create_smart_collection()
- add_tags()

All commands work exactly as before.

---

## BENEFITS

✅ **59% fewer components** - easier to maintain  
✅ **Cleaner code** - single responsibility  
✅ **Better performance** - less overhead  
✅ **Better UX** - consistent design  
✅ **Better DX** - easier to add features  
✅ **No backend changes** - zero risk  
✅ **Faster development** - less code to write  

---

## RISKS

| Risk | Mitigation |
|------|-----------|
| Breaking existing features | Test thoroughly before deleting |
| Performance regression | Monitor bundle size |
| State management bugs | Test stores extensively |
| UI inconsistency | Use design system |

---

## SUCCESS CRITERIA

- [ ] All 12 components working
- [ ] All 4 pages working
- [ ] All Tauri commands working
- [ ] No console errors
- [ ] No performance regression
- [ ] All features working
- [ ] Tests passing

---

## NEXT STEPS

1. **Review** this plan
2. **Approve** the redesign
3. **Start** Week 1 (Core components)
4. **Track** progress weekly
5. **Test** thoroughly
6. **Deploy** when ready

---

**Status**: READY TO START  
**Confidence**: HIGH  
**Risk**: LOW  

