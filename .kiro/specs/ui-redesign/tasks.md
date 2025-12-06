# Implementation Plan: TorrentFlix UI/UX Redesign

## Phase 1: Foundation & Core Components

- [x] 1. Set up TailwindCSS 4 and Svelte 5 upgrade
  - [x] 1.1 Update tailwind.config.js to v4 syntax
  - [x] 1.2 Update Svelte to v5 with runes support
  - [x] 1.3 Update TypeScript to 5.5+
  - [x] 1.4 Verify all dependencies compatible
  - _Requirements: 1.4, 24.1-24.6_

- [x] 2. Create core Button component
  - [x] 2.1 Implement variants: primary, secondary, ghost, danger
  - [x] 2.2 Implement sizes: sm, md, lg
  - [x] 2.3 Implement states: default, hover, active, disabled, loading
  - [x] 2.4 Add proper color contrast (4.5:1 minimum)
  - [x] 2.5 Write property tests for Button component
    - **Property 4: Interactive Element States**
    - **Validates: Requirements 1.5**
  - _Requirements: 1.1, 1.2, 1.5_

- [x] 3. Create core Input component
  - [x] 3.1 Implement types: text, password, email, number, search
  - [x] 3.2 Add password toggle functionality
  - [x] 3.3 Add clear button when text present
  - [x] 3.4 Implement validation feedback display
  - [x] 3.5 Ensure 4.5:1 contrast for input text
  - [x] 3.6 Write property tests for Input component
    - **Property 3: Form Input Contrast**
    - **Property 31: Real-time Validation**
    - **Property 32: Error Message Display**
    - **Validates: Requirements 1.3, 6.1, 6.2**
  - _Requirements: 1.1, 1.3, 5.6, 6.1, 6.2, 7.5, 7.6_

- [x] 4. Create core Modal component
  - [x] 4.1 Implement backdrop blur (glassmorphism)
  - [x] 4.2 Implement focus trap
  - [x] 4.3 Implement escape key close
  - [x] 4.4 Implement outside click close
  - [x] 4.5 Add smooth 300ms animations
  - [x] 4.6 Write property tests for Modal component
    - **Property 19: Modal Close Behavior**
    - **Property 20: Modal Animation**
    - **Property 55: Modal Focus Trap**
    - **Validates: Requirements 3.11, 3.12, 12.5**
  - _Requirements: 3.1, 3.12, 3.13, 24.3_

- [x] 5. Create core Card, Badge, Loading, Toast, Tabs components
  - [x] 5.1 Card: hover scale (1.05x), rounded corners (8px), shadow effects
  - [x] 5.2 Badge: variants (success, error, warning, info), sizes
  - [x] 5.3 Loading: types (spinner, skeleton, dots, pulse), animations
  - [x] 5.4 Toast: types, auto-dismiss (5s), manual dismiss, stacking
  - [x] 5.5 Tabs: keyboard navigation, smooth transitions
  - [x] 5.6 Write property tests for all components
    - **Property 11: Tile Border Radius**
    - **Property 40: Skeleton Placeholders**
    - **Property 41: Toast Notifications**
    - **Property 42: Toast Auto-dismiss**
    - **Property 50: Reduced Motion Respect**
    - **Validates: Requirements 2.7, 9.1, 9.3-9.5, 11.5**
  - _Requirements: 2.7, 9.1-9.6, 11.2, 11.3, 11.5, 12.2_

- [x] 6. Checkpoint - Ensure all core components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: Layout Components

- [x] 7. Create Header component
  - [x] 7.1 Fixed height 60px with glassmorphism effect
  - [x] 7.2 Centered search input (300-600px width)
  - [x] 7.3 Notification bell icon
  - [x] 7.4 Logo on left
  - [x] 7.5 Write property tests
    - **Property 26: Search Input Centering**
    - **Property 27: Search Input Width**
    - **Property 73: Header Glassmorphism**
    - **Validates: Requirements 5.2, 5.3, 24.1**
  - _Requirements: 5.1-5.7, 24.1_

- [x] 8. Create Sidebar component
  - [x] 8.1 Collapsible to icons only (200px → 60px)
  - [x] 8.2 Tooltips on hover when collapsed
  - [x] 8.3 Active page highlighting
  - [x] 8.4 Keyboard navigation (arrow keys)
  - [x] 8.5 Mobile hidden by default
  - [x] 8.6 Write property tests
    - **Property 21-25: Sidebar behaviors**
    - **Validates: Requirements 4.1-4.6**
  - _Requirements: 4.1-4.6_

- [x] 9. Create MainLayout component
  - [x] 9.1 Orchestrate Header + Sidebar + Content
  - [x] 9.2 Mobile menu toggle
  - [x] 9.3 Keyboard shortcuts (Escape, Ctrl+K)
  - [x] 9.4 Responsive layout
  - [x] 9.5 Write property tests
    - **Property 68: Ctrl+K Search Focus**
    - **Property 69: Escape Close Modal**
    - **Validates: Requirements 19.1, 19.2**
  - _Requirements: 4.6, 5.4, 5.5, 19.1, 19.2_

- [x] 10. Checkpoint - Ensure all layout components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Movie Tile & Detail Panel

- [x] 11. Create MovieTile component
  - [x] 11.1 2:3 aspect ratio (portrait)
  - [x] 11.2 Poster display with object-fit cover
  - [x] 11.3 Placeholder for missing poster
  - [x] 11.4 Title truncation with ellipsis
  - [x] 11.5 Ratings display (★ KP X.X | IMDB X.X | TMDB X.X)
  - [x] 11.6 Series info display (Season X • Y episodes)
  - [x] 11.7 Rounded corners (8px)
  - [x] 11.8 Write property tests
    - **Property 5-11: Tile properties**
    - **Validates: Requirements 2.1-2.7**
  - _Requirements: 2.1-2.8_

- [x] 12. Create DetailPanel component
  - [x] 12.1 Modal overlay with backdrop blur
  - [x] 12.2 Backdrop image (TMDB or poster fallback)
  - [x] 12.3 Movie title, description, duration
  - [x] 12.4 Cast display with photos
  - [x] 12.5 Ratings (KP, IMDB, TMDB) horizontal layout
  - [x] 12.6 Torrent info (seeders, leechers, file size)
  - [x] 12.7 Quality versions list (sorted by resolution)
  - [x] 12.8 Download and Bookmark buttons
  - [x] 12.9 Series info display
  - [x] 12.10 Close on outside click or Escape
  - [x] 12.11 300ms animation
  - [x] 12.12 Write property tests
    - **Property 12-20: Detail panel properties**
    - **Validates: Requirements 3.1-3.14**
  - _Requirements: 3.1-3.14_

- [ ] 13. Implement title normalization
  - [x] 13.1 Remove release group tags, quality indicators, encoding
  - [x] 13.2 Apply proper capitalization
  - [x] 13.3 Extract and display year separately
  - [x] 13.4 Handle special characters and unicode
  - [x] 13.5 Format series as S01E01
  - [x] 13.6 Write property tests
    - **Property 56-58: Title normalization**
    - **Validates: Requirements 13.1, 13.2, 13.5**
  - _Requirements: 13.1-13.5_

- [x] 14. Checkpoint - Ensure all tile/detail components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Page Components

- [x] 15. Create Discover page
  - [x] 15.1 Movie grid with responsive columns (2-6 based on viewport)
  - [x] 15.2 Infinite scroll or Load More button (configurable)
  - [x] 15.3 Feed caching in memory
  - [x] 15.4 Search results caching
  - [x] 15.5 Scroll position preservation
  - [x] 15.6 Manual refresh button
  - [x] 15.7 10-minute cache invalidation
  - [x] 15.8 Detail panel modal on tile click
  - [x] 15.9 Write property tests
    - **Property 37-39, 61-67: Grid and cache properties**
    - **Validates: Requirements 8.1-8.7, 15.1-15.7, 18.1-18.6**
  - _Requirements: 8.1-8.7, 15.1-15.7, 16.1-16.6, 18.1-18.6_

- [x] 16. Create Library, Downloads, Settings pages
  - [x] 16.1 Library: Tabbed interface (Browser, Collections, Analytics, Cleanup, Jobs)
  - [x] 16.2 Downloads: Active downloads list, progress bars, controls
  - [x] 16.3 Settings: Grouped sections, validation, notifications
  - [x] 16.4 Write property tests for Settings
    - **Property 31-36, 43-46, 71-72: Settings properties**
    - **Validates: Requirements 6.1-6.7, 10.1-10.5, 20.1-20.5**
  - _Requirements: 6.1-6.7, 10.1-10.5, 20.1-20.5_

- [x] 17. Checkpoint - Ensure all page components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Advanced Features

- [x] 18. Implement search functionality
  - [x] 18.1 Debounce 300ms on input
  - [x] 18.2 Immediate search on Enter key
  - [x] 18.3 Clear button when text present
  - [x] 18.4 Search history (up to 10 items)
  - [x] 18.5 Autocomplete suggestions (debounce 150ms)
  - [x] 18.6 Persist search history to localStorage
  - [x] 18.7 Write property tests
    - **Property 28-30: Search properties**
    - **Validates: Requirements 5.4-5.6**
  - _Requirements: 5.4-5.6, 17.1-17.6_

- [ ] 19. Implement keyboard shortcuts & animations
  - [x] 19.1 Ctrl/Cmd+K: focus search
  - [x] 19.2 Escape: close modals
  - [x] 19.3 Arrow keys: navigate tiles
  - [x] 19.4 Enter: open detail panel
  - [x] 19.5 Number keys 1-4: switch pages
  - [x] 19.6 Page navigation: 200-300ms fade/slide
  - [x] 19.7 Hover transitions: 150ms
  - [x] 19.8 Tile hover scale: 1.05x
  - [x] 19.9 Detail panel slide: 300ms
  - [x] 19.10 Respect prefers-reduced-motion
  - [x] 19.11 Write property tests
    - **Property 47-50, 68-70: Animation and shortcut properties**
    - **Validates: Requirements 11.1-11.5, 19.1-19.6**
  - _Requirements: 11.1-11.5, 19.1-19.6_

- [x] 20. Implement accessibility features
  - [x] 20.1 ARIA labels for all interactive elements
  - [x] 20.2 Full keyboard navigation
  - [x] 20.3 Visible focus indicators
  - [x] 20.4 Semantic HTML (nav, main, header, button)
  - [x] 20.5 Focus trap in modals
  - [x] 20.6 ARIA live regions for dynamic content
  - [x] 20.7 Write property tests
    - **Property 51-55: Accessibility properties**
    - **Validates: Requirements 12.1-12.6**
  - _Requirements: 12.1-12.6_

- [x] 21. Implement quality selection & pagination
  - [x] 21.1 List all available quality versions
  - [x] 21.2 Display resolution, file size, seeders, leechers
  - [x] 21.3 Sort by resolution (highest first)
  - [x] 21.4 Highlight recommended option
  - [x] 21.5 Make scrollable if >5 options
  - [x] 21.6 Settings option: Infinite Scroll or Load More Button
  - [x] 21.7 Infinite scroll: auto-load at 200px from bottom
  - [x] 21.8 Load More button: manual trigger
  - [x] 21.9 Loading indicator during load
  - [x] 21.10 Prevent duplicate API calls
  - [x] 21.11 End of results message
  - [x] 21.12 Persist preference across sessions
  - [x] 21.13 Write property tests
    - **Property 59-63: Quality and pagination properties**
    - **Validates: Requirements 14.1-14.6, 15.1-15.7**
  - _Requirements: 14.1-14.6, 15.1-15.7_

- [x] 22. Implement theme & localization
  - [x] 22.1 Dark theme (default)
  - [x] 22.2 OLED theme (darker)
  - [x] 22.3 Accent color selection (red, blue, green, purple, orange)
  - [x] 22.4 Persist preferences across sessions
  - [x] 22.5 Apply changes immediately without reload
  - [x] 22.6 Theme preview section in settings
  - [x] 22.7 Load i18n on startup
  - [x] 22.8 Support English and Russian
  - [x] 22.9 Fallback to key name for missing translations
  - [x] 22.10 Persist language preference
  - [x] 22.11 Update UI without reload on language change
  - [x] 22.12 Write property tests
    - **Property 43-46, 71-72: Theme and localization properties**
    - **Validates: Requirements 10.1-10.5, 20.1-20.5**
  - _Requirements: 10.1-10.5, 20.1-20.5_

- [ ] 23. Implement modern effects & color contrast
  - [x] 23.1 Header glassmorphism (backdrop blur)
  - [x] 23.2 Sidebar subtle gradients
  - [x] 23.3 Modal backdrop blur
  - [x] 23.4 Subtle shadow effects on cards
  - [x] 23.5 Gradient overlays on tile hover
  - [x] 23.6 Opacity-based separators (no harsh borders)
  - [x] 23.7 Verify body text contrast (4.5:1)
  - [x] 23.8 Verify large text contrast (3:1)
  - [x] 23.9 Verify form input contrast (4.5:1)
  - [x] 23.10 Verify interactive element states
  - [x] 23.11 Write property tests
    - **Property 1-4, 73-74: Contrast and effects properties**
    - **Validates: Requirements 1.1-1.5, 24.1-24.6**
  - _Requirements: 1.1-1.5, 24.1-24.6_

- [x] 24. Checkpoint - Ensure all advanced features pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Polish & Optimization

- [x] 25. Implement empty states, context menus, notifications
  - [x] 25.1 Empty library state with guidance
  - [x] 25.2 No search results with suggestions
  - [x] 25.3 Empty downloads state with explanation
  - [x] 25.4 Action buttons in empty states
  - [x] 25.5 Setup wizard prompt for incomplete settings
  - [x] 25.6 Right-click context menus on tiles
  - [x] 25.7 Context menu options: View Details, Download, Add to Collection, Copy Title
  - [x] 25.8 Download item context menu: Pause, Resume, Cancel, Open Folder
  - [x] 25.9 Context menu close on outside click or Escape
  - [x] 25.10 Context menu keyboard navigation
  - [x] 25.11 Notification bell icon in header
  - [x] 25.12 Unread count badge
  - [x] 25.13 Notification dropdown with recent notifications (up to 50)
  - [x] 25.14 Timestamp, type icon, message display
  - [x] 25.15 Mark as read when viewed
  - [x] 25.16 Clear All option
  - _Requirements: 21.1-21.5, 22.1-22.5, 23.1-23.7_

- [x] 26. Performance optimization & accessibility audit
  - [x] 26.1 Lazy load images
  - [x] 26.2 Code splitting for pages
  - [x] 26.3 Optimize bundle size
  - [x] 26.4 Monitor grid rendering with 100+ tiles
  - [x] 26.5 Verify animations run at 60fps
  - [x] 26.6 WCAG AA compliance check
  - [x] 26.7 Keyboard navigation full test
  - [x] 26.8 Screen reader compatibility test
  - [x] 26.9 Focus management verification
  - [x] 26.10 ARIA labels audit
  - _Requirements: 8.1-8.7, 12.1-12.6_

- [x] 27. Final integration testing
  - [x] 27.1 Test all page navigation
  - [x] 27.2 Test modal flows
  - [x] 27.3 Test search and filtering
  - [x] 27.4 Test settings save/load
  - [x] 27.5 Test keyboard shortcuts
  - [x] 27.6 Test responsive design on multiple devices
  - _Requirements: All_

- [x] 28. Checkpoint - Final verification
  - Ensure all tests pass, ask the user if questions arise.

## Phase 7: Cleanup & Deprecation
- [x] 29. Remove old components (17 components)
  - [x] 29.1 Delete TopBar, TabBar, MovieGrid, MovieCard, MovieTile
  - [x] 29.2 Delete DetailPanel (old), DownloadsTab, SettingsTab
  - [x] 29.3 Delete LibraryBrowser, MediaItemDetail, CollectionManager
  - [x] 29.4 Delete StorageAnalytics, ManagementPanel, JobsDashboard
  - [x] 29.5 Delete CleanupManager, VersionManager, SkeletonTile
  - [x] 29.6 Delete LoadingSpinner, LoadingDots, SearchBar, SecureInput
  - [x] 29.7 Delete SettingsSection, SettingsNotification

- [x] 30. Update App.svelte & final deployment
  - [x] 30.1 Remove old imports
  - [x] 30.2 Update to use new MainLayout
  - [x] 30.3 Verify all pages work
  - [x] 30.4 Clean up old state management
  - [x] 30.5 Build production bundle
  - [x] 30.6 Verify no console errors
  - [x] 30.7 Test on target platforms
  - [x] 30.8 Deploy to users
