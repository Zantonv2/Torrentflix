# Implementation Plan: TorrentFlix UI/UX Redesign

## Phase 1: Foundation & Core Components

- [ ] 1. Set up TailwindCSS 4 and Svelte 5 upgrade
  - Update tailwind.config.js to v4 syntax
  - Update Svelte to v5 with runes support
  - Update TypeScript to 5.5+
  - Verify all dependencies compatible
  - _Requirements: 1.4, 24.1-24.6_

- [ ] 2. Create core Button component
  - Implement variants: primary, secondary, ghost, danger
  - Implement sizes: sm, md, lg
  - Implement states: default, hover, active, disabled, loading
  - Add proper color contrast (4.5:1 minimum)
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 2.1 Write property tests for Button component
  - **Property 4: Interactive Element States**
  - **Validates: Requirements 1.5**

- [ ] 3. Create core Input component
  - Implement types: text, password, email, number, search
  - Add password toggle functionality
  - Add clear button when text present
  - Implement validation feedback display
  - Ensure 4.5:1 contrast for input text
  - _Requirements: 1.1, 1.3, 5.6, 6.1, 6.2, 7.5, 7.6_

- [ ] 3.1 Write property tests for Input component
  - **Property 3: Form Input Contrast**
  - **Property 31: Real-time Validation**
  - **Property 32: Error Message Display**
  - **Validates: Requirements 1.3, 6.1, 6.2**

- [ ] 4. Create core Modal component
  - Implement backdrop blur (glassmorphism)
  - Implement focus trap
  - Implement escape key close
  - Implement outside click close
  - Add smooth 300ms animations
  - _Requirements: 3.1, 3.12, 3.13, 24.3_

- [ ] 4.1 Write property tests for Modal component
  - **Property 19: Modal Close Behavior**
  - **Property 20: Modal Animation**
  - **Property 55: Modal Focus Trap**
  - **Validates: Requirements 3.11, 3.12, 12.5**

- [ ] 5. Create core Card component
  - Implement hover scale (1.05x)
  - Implement rounded corners (8px minimum)
  - Implement shadow effects
  - Add smooth transitions (150ms)
  - _Requirements: 2.7, 11.2, 11.3, 24.4_

- [ ] 5.1 Write property tests for Card component
  - **Property 11: Tile Border Radius**
  - **Property 49: Tile Hover Scale**
  - **Validates: Requirements 2.7, 11.3**

- [ ] 6. Create core Badge component
  - Implement variants: success, error, warning, info
  - Implement sizes: sm, md, lg
  - Ensure proper color contrast
  - _Requirements: 1.1, 1.2_

- [ ] 7. Create core Loading component
  - Implement types: spinner, skeleton, dots, pulse
  - Add smooth animations
  - Respect prefers-reduced-motion
  - _Requirements: 9.1, 9.2, 11.5_

- [ ] 7.1 Write property tests for Loading component
  - **Property 40: Skeleton Placeholders**
  - **Property 50: Reduced Motion Respect**
  - **Validates: Requirements 9.1, 11.5**

- [ ] 8. Create core Toast component
  - Implement types: success, error, warning, info
  - Auto-dismiss after 5 seconds
  - Manual dismiss button
  - Stacking support
  - _Requirements: 9.3, 9.4, 9.5, 9.6_

- [ ] 8.1 Write property tests for Toast component
  - **Property 41: Toast Notifications**
  - **Property 42: Toast Auto-dismiss**
  - **Validates: Requirements 9.3, 9.4, 9.5**

- [ ] 9. Create core Tabs component
  - Keyboard navigation support
  - Smooth transitions
  - Active tab highlighting
  - _Requirements: 12.2_

- [ ] 10. Checkpoint - Ensure all core components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 2: Layout Components

- [ ] 11. Create Header component
  - Fixed height 60px
  - Glassmorphism effect (backdrop blur)
  - Centered search input (300-600px width)
  - Notification bell icon
  - Logo on left
  - _Requirements: 5.1, 5.2, 5.3, 5.7, 24.1_

- [ ] 11.1 Write property tests for Header
  - **Property 26: Search Input Centering**
  - **Property 27: Search Input Width**
  - **Property 73: Header Glassmorphism**
  - **Validates: Requirements 5.2, 5.3, 24.1**

- [ ] 12. Create Sidebar component
  - Collapsible to icons only (200px → 60px)
  - Tooltips on hover when collapsed
  - Active page highlighting
  - Keyboard navigation (arrow keys)
  - Mobile hidden by default
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [ ] 12.1 Write property tests for Sidebar
  - **Property 21: Sidebar Collapse**
  - **Property 22: Sidebar Tooltips**
  - **Property 23: Active Page Highlight**
  - **Property 24: Keyboard Navigation**
  - **Property 25: Mobile Sidebar Hidden**
  - **Validates: Requirements 4.1, 4.2, 4.4, 4.5, 4.6**

- [ ] 13. Create MainLayout component
  - Orchestrate Header + Sidebar + Content
  - Mobile menu toggle
  - Keyboard shortcuts (Escape, Ctrl+K)
  - Responsive layout
  - _Requirements: 4.6, 5.4, 5.5, 19.1, 19.2_

- [ ] 13.1 Write properrty tests fr MainLayout
  - **Property 68: Ctrl+K Search Focus**
  - **Property 69: Escape Close Modal**
  - **Validates: Requirements 19.1, 19.2**

- [ ] 14. Checkpoint - Ensure all layout components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 3: Movie Tile & Detail Panel

- [ ] 15. Create MovieTile component
  - 2:3 aspect ratio (portrait)
  - Poster display with object-fit cover
  - Placeholder for missing poster
  - Title truncation with ellipsis
  - Ratings display (★ KP X.X | IMDB X.X | TMDB X.X)
  - Series info display (Season X • Y episodes)
  - Rounded corners (8px)
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8_

- [ ] 15.1 Write property tests for MovieTile
  - **Property 5: Tile Aspect Ratio**
  - **Property 6: Poster Display**
  - **Property 7: Missing Poster Placeholder**
  - **Property 8: Title Truncation**
  - **Property 9: Ratings Format**
  - **Property 10: Series Info Display**
  - **Property 11: Tile Border Radius**
  - **Validates: Requirements 2.1-2.7**

- [ ] 16. Create DetailPanel component
  - Modal overlay with backdrop blur
  - Backdrop image (TMDB or poster fallback)
  - Movie title, description, duration
  - Cast display with photos
  - Ratings (KP, IMDB, TMDB) horizontal layout
  - Torrent info (seeders, leechers, file size)
  - Quality versions list (sorted by resolution)
  - Download and Bookmark buttons
  - Series info display
  - Close on outside click or Escape
  - 300ms animation
  - _Requirements: 3.1-3.14_

- [ ] 16.1 Write property tests for DetailPanel
  - **Property 12: Modal Opens on Click**
  - **Property 13: Backdrop Image Display**
  - **Property 14: Detail Panel Content**
  - **Property 15: Cast Display**
  - **Property 16: Ratings Display**
  - **Property 17: Torrent Info Display**
  - **Property 18: Quality Versions List**
  - **Property 19: Modal Close Behavior**
  - **Property 20: Modal Animation**
  - **Validates: Requirements 3.1-3.14**

- [ ] 17. Implement title normalization
  - Remove release group tags
  - Remove quality indicators
  - Remove encoding information
  - Apply proper capitalization
  - Extract and display year separately
  - Handle special characters and unicode
  - Format series as S01E01
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_

- [ ] 17.1 Write property tests for title normalization
  - **Property 56: Title Normalization**
  - **Property 57: Title Capitalization**
  - **Property 58: Series Format**
  - **Validates: Requirements 13.1, 13.2, 13.5**

- [ ] 18. Checkpoint - Ensure all tile/detail components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 4: Page Components

- [ ] 19. Create Discover page
  - Movie grid with responsive columns (2-6 based on viewport)
  - Infinite scroll or Load More button (configurable)
  - Feed caching in memory
  - Search results caching
  - Scroll position preservation
  - Manual refresh button
  - 10-minute cache invalidation
  - Detail panel modal on tile click
  - _Requirements: 8.1-8.7, 15.1-15.7, 16.1-16.6, 18.1-18.6_

- [ ] 19.1 Write property tests for Discover page
  - **Property 37: Grid Responsiveness**
  - **Property 38: Grid Breakpoints**
  - **Property 39: Grid Gap Spacing**
  - **Property 61: Infinite Scroll Auto-load**
  - **Property 62: Load More Button**
  - **Property 64: Feed Caching**
  - **Property 65: Cache Persistence**
  - **Property 66: Scroll Position Preservation**
  - **Property 67: Cache Invalidation**
  - **Validates: Requirements 8.1-8.7, 15.1-15.7, 18.1-18.6**

- [ ] 20. Create Library page
  - Tabbed interface (Browser, Collections, Analytics, Cleanup, Jobs)
  - Search functionality
  - Filtering and sorting
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 21. Create Downloads page
  - Active downloads list
  - Progress bars
  - Pause, resume, cancel buttons
  - Open folder option
  - Polling every 2 seconds
  - _Requirements: 1.1, 1.2, 1.5_

- [ ] 22. Create Settings page
  - Grouped sections (API Keys, qBittorrent, Paths, Appearance, Pagination)
  - Real-time validation
  - Success/error notifications
  - Theme customization (dark, OLED)
  - Accent color selection (red, blue, green, purple, orange)
  - Language selection (English, Russian)
  - Pagination mode selection (infinite, button)
  - Password toggle for sensitive fields
  - _Requirements: 6.1-6.7, 10.1-10.5, 20.1-20.5_

- [ ] 22.1 Write property tests for Settings page
  - **Property 31: Real-time Validation**
  - **Property 32: Error Message Display**
  - **Property 33: Save Validation**
  - **Property 34: Success Notification**
  - **Property 35: Checkbox Size**
  - **Property 36: Checkmark Contrast**
  - **Property 43: Localization Load**
  - **Property 44: Language Support**
  - **Property 45: Missing Translation Fallback**
  - **Property 46: Language Persistence**
  - **Property 71: Theme Persistence**
  - **Property 72: Theme Change Immediate**
  - **Validates: Requirements 6.1-6.7, 10.1-10.5, 20.1-20.5**

- [ ] 23. Checkpoint - Ensure all page components pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 5: Advanced Features

- [ ] 24. Implement search functionality
  - Debounce 300ms on input
  - Immediate search on Enter key
  - Clear button when text present
  - Search history (up to 10 items)
  - Autocomplete suggestions (debounce 150ms)
  - Persist search history to localStorage
  - Clear all option
  - _Requirements: 5.4, 5.5, 5.6, 17.1-17.6_

- [ ] 24.1 Write property tests for search
  - **Property 28: Search Debounce**
  - **Property 29: Enter Key Search**
  - **Property 30: Clear Button Display**
  - **Validates: Requirements 5.4, 5.5, 5.6**

- [ ] 25. Implement keyboard shortcuts
  - Ctrl/Cmd+K: focus search
  - Escape: close modals
  - Arrow keys: navigate tiles
  - Enter: open detail panel
  - Number keys 1-4: switch pages
  - Display shortcut hints in tooltips
  - _Requirements: 19.1-19.6_

- [ ] 25.1 Write property tests for keyboard shortcuts
  - **Property 68: Ctrl+K Search Focus**
  - **Property 69: Escape Close Modal**
  - **Property 70: Arrow Key Navigation**
  - **Validates: Requirements 19.1, 19.2, 19.3**

- [ ] 26. Implement animations and transitions
  - Page navigation: 200-300ms fade/slide
  - Hover transitions: 150ms
  - Tile hover scale: 1.05x
  - Detail panel slide: 300ms
  - Respect prefers-reduced-motion
  - _Requirements: 11.1-11.5_

- [ ] 26.1 Write property tests for animations
  - **Property 47: Page Navigation Animation**
  - **Property 48: Hover Transitions**
  - **Property 49: Tile Hover Scale**
  - **Property 50: Reduced Motion Respect**
  - **Validates: Requirements 11.1-11.5**

- [ ] 27. Implement accessibility features
  - ARIA labels for all interactive elements
  - Full keyboard navigation
  - Visible focus indicators
  - Semantic HTML (nav, main, header, button)
  - Focus trap in modals
  - ARIA live regions for dynamic content
  - _Requirements: 12.1-12.6_

- [ ] 27.1 Write property tests for accessibility
  - **Property 51: ARIA Labels**
  - **Property 52: Keyboard Navigation**
  - **Property 53: Focus Indicators**
  - **Property 54: Semantic HTML**
  - **Property 55: Modal Focus Trap**
  - **Validates: Requirements 12.1-12.6**

- [ ] 28. Implement quality selection
  - List all available quality versions
  - Display resolution, file size, seeders, leechers
  - Sort by resolution (highest first)
  - Highlight recommended option
  - Make scrollable if >5 options
  - Download selected quality
  - _Requirements: 14.1-14.6_

- [ ] 28.1 Write property tests for quality selection
  - **Property 59: Quality Sorting**
  - **Property 60: Recommended Quality**
  - **Validates: Requirements 14.3, 14.5**

- [ ] 29. Implement pagination options
  - Settings option: Infinite Scroll or Load More Button
  - Infinite scroll: auto-load at 200px from bottom
  - Load More button: manual trigger
  - Loading indicator during load
  - Prevent duplicate API calls
  - End of results message
  - Persist preference across sessions
  - _Requirements: 15.1-15.7_

- [ ] 29.1 Write property tests for pagination
  - **Property 61: Infinite Scroll Auto-load**
  - **Property 62: Load More Button**
  - **Property 63: Pagination Preference Persistence**
  - **Validates: Requirements 15.2, 15.3, 15.7**

- [ ] 30. Implement theme customization
  - Dark theme (default)
  - OLED theme (darker)
  - Accent color selection (red, blue, green, purple, orange)
  - Persist preferences across sessions
  - Apply changes immediately without reload
  - Theme preview section in settings
  - _Requirements: 20.1-20.5_

- [ ] 30.1 Write property tests for theme
  - **Property 71: Theme Persistence**
  - **Property 72: Theme Change Immediate**
  - **Validates: Requirements 20.3, 20.4**

- [ ] 31. Implement localization
  - Load i18n on startup
  - Support English and Russian
  - Fallback to key name for missing translations
  - Persist language preference
  - Update UI without reload on language change
  - _Requirements: 10.1-10.5_

- [ ]* 31.1 Write property tests for localization
  - **Property 43: Localization Load**
  - **Property 44: Language Support**
  - **Property 45: Missing Tr anslation Fallback**
  - **Property 46: Language Persistence**
  - **Validates: Requirements 10.1-10.5**

- [ ] 32. Implement modern effects
  - Header glassmorphism (backdrop blur)
  - Sidebar subtle gradients
  - Modal backdrop blur
  - Subtle shadow effects on cards
  - Gradient overlays on tile hover
  - Opacity-based separators (no harsh borders)
  - _Requirements: 24.1-24.6_

- [ ] 32.1 Write property tests for modern effects
  - **Property 73: Header Glassmorphism**
  - **Property 74: Modal Backdrop Blur**
  - **Validates: Requirements 24.1, 24.3**

- [ ] 33. Checkpoint - Ensure all advanced features pass tests
  - Ensure all tests pass, ask the user if questions arise.

## Phase 6: Polish & Optimization

- [ ] 34. Implement empty states
  - Empty library state with guidance
  - No search results with suggestions
  - Empty downloads state with explanation
  - Action buttons in empty states
  - Setup wizard prompt for incomplete settings
  - _Requirements: 21.1-21.5_

- [ ] 35. Implement context menus
  - Right-click on movie tiles
  - Options: View Details, Download, Add to Collection, Copy Title
  - Right-click on download items
  - Options: Pause, Resume, Cancel, Open Folder
  - Close on outside click or Escape
  - Keyboard navigation with arrow keys
  - _Requirements: 22.1-22.5_

- [ ] 36. Implement notification center
  - Notification bell icon in header
  - Unread count badge
  - Dropdown with recent notifications (up to 50)
  - Timestamp, type icon, message display
  - Mark as read when viewed
  - Clear All option
  - _Requirements: 23.1-23.7_

- [ ] 37. Implement color contrast verification
  - Verify all text meets 4.5:1 contrast (body text)
  - Verify all large text meets 3:1 contrast
  - Verify form inputs meet 4.5:1 contrast
  - Verify interactive elements have distinct states
  - _Requirements: 1.1-1.5_

- [ ] 37.1 Write property tests for color contrast
  - **Property 1: Body Text Contrast**
  - **Property 2: Large Text Contrast**
  - **Property 3: Form Input Contrast**
  - **Property 4: Interactive Element States**
  - **Validates: Requirements 1.1-1.5**

- [ ] 38. Performance optimization
  - Lazy load images
  - Code splitting for pages
  - Optimize bundle size
  - Monitor grid rendering with 100+ tiles
  - Verify animations run at 60fps
  - _Requirements: 8.1-8.7_

- [ ] 39. Accessibility audit
  - WCAG AA compliance check
  - Keyboard navigation full test
  - Screen reader compatibility test
  - Focus management verification
  - ARIA labels audit
  - _Requirements: 12.1-12.6_

- [ ] 40. Final integration testing
  - Test all page navigation
  - Test modal flows
  - Test search and filtering
  - Test settings save/load
  - Test keyboard shortcuts
  - Test responsive design on multiple devices
  - _Requirements: All_

- [ ] 41. Checkpoint - Final verification
  - Ensure all tests pass, ask the user if questions arise.

## Cleanup & Deprecation

- [ ] 42. Remove old components
  - Delete TopBar.svelte
  - Delete TabBar.svelte
  - Delete MovieGrid.svelte (consolidated into Discover)
  - Delete MovieCard.svelte
  - Delete MovieTile.svelte
  - Delete DetailPanel.svelte (old version)
  - Delete DownloadsTab.svelte
  - Delete SettingsTab.svelte
  - Delete LibraryBrowser.svelte
  - Delete MediaItemDetail.svelte
  - Delete CollectionManager.svelte
  - Delete StorageAnalytics.svelte
  - Delete ManagementPanel.svelte
  - Delete JobsDashboard.svelte
  - Delete CleanupManager.svelte
  - Delete VersionManager.svelte
  - Delete SkeletonTile.svelte
  - Delete LoadingSpinner.svelte
  - Delete LoadingDots.svelte
  - Delete SearchBar.svelte
  - Delete SecureInput.svelte
  - Delete SettingsSection.svelte
  - Delete SettingsNotification.svelte

- [ ] 43. Update App.svelte
  - Remove old imports
  - Update to use new MainLayout
  - Verify all pages work
  - Clean up old state management

- [ ] 44. Final deployment
  - Build production bundle
  - Verify no console errors
  - Test on target platforms
  - Deploy to users
