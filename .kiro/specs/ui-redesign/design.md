# Design Document: TorrentFlix UI/UX Redesign

## Overview

This document outlines the technical design for a comprehensive UI/UX redesign of TorrentFlix using Svelte 5 and TailwindCSS 4. The redesign transforms the application from a dated, component-heavy interface into a modern, Netflix-inspired streaming service with improved accessibility, performance, and user experience.

**Key Goals:**
- Fix color contrast and accessibility issues
- Implement proper 2:3 movie tile aspect ratios
- Create comprehensive detail panels with full metadata
- Add collapsible sidebar navigation
- Improve responsive design across all breakpoints
- Implement modern glassmorphism effects
- Reduce component count from 29 to 12 core components

## Architecture

### Component Hierarchy

```
App.svelte (root)
├── MainLayout
│   ├── Header
│   │   ├── Logo
│   │   ├── SearchInput
│   │   └── NotificationBell
│   ├── Sidebar
│   │   ├── NavItems
│   │   └── CollapseToggle
│   └── Content (page router)
│       ├── Discover
│       ├── Library
│       ├── Downloads
│       └── Settings
├── Core Components (reusable)
│   ├── Button
│   ├── Input
│   ├── Modal
│   ├── Card
│   ├── Badge
│   ├── Loading
│   ├── Toast
│   └── Tabs
└── Stores (state management)
    ├── searchStore
    ├── libraryStore
    ├── downloadStore
    ├── settingsStore
    └── uiStore
```

### Data Flow

1. **UI Layer** (Svelte components) → dispatches events
2. **Store Layer** (Svelte stores) → manages state
3. **Tauri Bridge** (commands.rs) → calls backend
4. **Engine** (Rust) → business logic
5. **Database** (SQLite) → persistence

## Components and Interfaces

### Core Components

**Button.svelte**
- Variants: primary, secondary, ghost, danger
- Sizes: sm, md, lg
- States: default, hover, active, disabled, loading
- Props: variant, size, disabled, loading, onClick

**Input.svelte**
- Types: text, password, email, number, search
- Props: type, value, placeholder, error, disabled, label
- Features: password toggle, clear button, validation feedback

**Modal.svelte**
- Props: open, title, onClose, size (sm/md/lg)
- Features: backdrop blur, focus trap, escape key close, outside click close
- Slots: header, body, footer

**Card.svelte**
- Props: hoverable, clickable, class
- Features: hover scale (1.05x), rounded corners (8px), shadow effects
- Slots: default

**Badge.svelte**
- Props: variant (success, error, warning, info), size
- Features: color-coded status indicators

**Loading.svelte**
- Types: spinner, skeleton, dots, pulse
- Props: type, size
- Features: smooth animations, respects prefers-reduced-motion

**Toast.svelte**
- Props: open, message, type (success, error, warning, info), duration
- Features: auto-dismiss (5s), manual dismiss, stacking

**Tabs.svelte**
- Props: tabs (array of {id, label}), activeTab
- Features: keyboard navigation, smooth transitions

### Layout Components

**Header.svelte**
- Fixed height: 60px
- Features: glassmorphism effect, centered search, notification bell
- Responsive: hamburger menu on mobile

**Sidebar.svelte**
- Width: 200px (expanded), 60px (collapsed)
- Features: collapsible, tooltips, keyboard navigation
- Responsive: hidden on mobile, accessible via hamburger

**MainLayout.svelte**
- Orchestrates Header + Sidebar + Content
- Manages mobile menu state
- Handles keyboard shortcuts (Escape, Ctrl+K)

### Page Components

**Discover.svelte**
- Displays movie grid with infinite scroll or load more
- Features: feed caching, search results caching, state persistence
- Detail panel modal on tile click

**Library.svelte**
- Tabbed interface: Browser, Collections, Analytics, Cleanup, Jobs
- Features: search, filtering, sorting

**Downloads.svelte**
- Active downloads list with progress bars
- Features: pause, resume, cancel, open folder
- Polling every 2 seconds

**Settings.svelte**
- Grouped sections: API Keys, qBittorrent, Paths, Appearance, Pagination
- Features: real-time validation, success/error notifications
- Theme customization, language selection

## Data Models

### Movie Tile Data
```typescript
interface MovieTile {
  id: string;
  title: string;           // normalized
  posterPath: string;      // TMDB poster URL
  ratings: {
    kinopoisk?: number;
    imdb?: number;
    tmdb?: number;
  };
  year: number;
  isSeriesContent: boolean;
  seriesInfo?: {
    season: number;
    episodeCount: number;
  };
}
```

### Detail Panel Data
```typescript
interface MovieDetail {
  id: string;
  title: string;           // normalized
  backdropPath: string;    // TMDB backdrop URL
  posterPath: string;      // fallback
  description: string;
  duration: number;        // minutes
  genres: string[];
  cast: Array<{
    name: string;
    character: string;
    profilePath: string;
  }>;
  ratings: {
    kinopoisk?: number;
    imdb?: number;
    tmdb?: number;
  };
  torrentInfo: {
    seeders: number;
    leechers: number;
    fileSize: string;
    qualities: Array<{
      resolution: string;
      fileSize: string;
      seeders: number;
      leechers: number;
      magnetLink: string;
    }>;
  };
  isSeriesContent: boolean;
  seriesInfo?: {
    season: number;
    episodeCount: number;
  };
}
```

### Store State
```typescript
// searchStore
{
  query: string;
  results: MovieTile[];
  loading: boolean;
  error: string | null;
  cache: Map<string, MovieTile[]>;
  cacheTimestamp: number;
}

// uiStore
{
  currentPage: 'discover' | 'library' | 'downloads' | 'settings';
  sidebarCollapsed: boolean;
  theme: 'dark' | 'oled';
  accentColor: 'red' | 'blue' | 'green' | 'purple' | 'orange';
  paginationMode: 'infinite' | 'button';
  language: 'en' | 'ru';
}
```

## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Color & Contrast Properties

**Property 1: Body Text Contrast**
*For any* text element with class "body-text", the contrast ratio between text color and background SHALL be at least 4.5:1
**Validates: Requirements 1.1**

**Property 2: Large Text Contrast**
*For any* text element with font-size >= 18px or (font-size >= 14px AND font-weight >= 600), the contrast ratio SHALL be at least 3:1
**Validates: Requirements 1.2**

**Property 3: Form Input Contrast**
*For any* form input element, the contrast ratio between input text and input background SHALL be at least 4.5:1
**Validates: Requirements 1.3**

**Property 4: Interactive Element States**
*For any* interactive element, hover and focus states SHALL have visually distinct colors from the default state
**Validates: Requirements 1.5**

### Movie Tile Properties

**Property 5: Tile Aspect Ratio**
*For any* movie tile, the aspect ratio of the poster image container SHALL be 2:3 (portrait)
**Validates: Requirements 2.1**

**Property 6: Poster Display**
*For any* movie with a poster URL, the poster image SHALL fill the tile area with object-fit: cover
**Validates: Requirements 2.2**

**Property 7: Missing Poster Placeholder**
*For any* movie without a poster URL, a placeholder with the movie title SHALL display in the tile
**Validates: Requirements 2.3**

**Property 8: Title Truncation**
*For any* movie title exceeding two lines, the title SHALL be truncated with ellipsis
**Validates: Requirements 2.4**

**Property 9: Ratings Format**
*For any* movie with ratings, ratings SHALL display in the format "★ KP X.X | IMDB X.X | TMDB X.X"
**Validates: Requirements 2.5**

**Property 10: Series Info Display**
*For any* TV series content, the tile SHALL display season number and episode count
**Validates: Requirements 2.6**

**Property 11: Tile Border Radius**
*For any* movie tile, the border-radius SHALL be at least 8px
**Validates: Requirements 2.7**

### Detail Panel Properties

**Property 12: Modal Opens on Click**
*For any* movie tile click, the detail panel modal SHALL open with the movie information
**Validates: Requirements 3.1**

**Property 13: Backdrop Image Display**
*For any* detail panel, a backdrop image SHALL display (from TMDB or poster as fallback)
**Validates: Requirements 3.2**

**Property 14: Detail Panel Content**
*For any* detail panel, all required fields (title, description, duration, genres) SHALL be present
**Validates: Requirements 3.3**

**Property 15: Cast Display**
*For any* movie with cast data, the top cast members with photos SHALL display in the detail panel
**Validates: Requirements 3.4**

**Property 16: Ratings Display**
*For any* detail panel, ratings from Kinopoisk, IMDb, and TMDB SHALL display in horizontal layout
**Validates: Requirements 3.5**

**Property 17: Torrent Info Display**
*For any* detail panel, seeders, leechers, and file size SHALL display
**Validates: Requirements 3.6**

**Property 18: Quality Versions List**
*For any* detail panel, all available quality versions from indexers SHALL be listed
**Validates: Requirements 3.7**

**Property 19: Modal Close Behavior**
*For any* detail panel, clicking outside or pressing Escape SHALL close the modal
**Validates: Requirements 3.11**

**Property 20: Modal Animation**
*For any* detail panel open/close, the animation SHALL complete in 300ms
**Validates: Requirements 3.12**

### Navigation Properties

**Property 21: Sidebar Collapse**
*For any* sidebar collapse toggle click, the sidebar SHALL animate to collapsed state showing only icons
**Validates: Requirements 4.1**

**Property 22: Sidebar Tooltips**
*For any* collapsed sidebar item hover, a tooltip with the item label SHALL display
**Validates: Requirements 4.2**

**Property 23: Active Page Highlight**
*For any* page, the corresponding sidebar item SHALL be highlighted with distinct visual indicator
**Validates: Requirements 4.4**

**Property 24: Keyboard Navigation**
*For any* sidebar, arrow keys SHALL navigate between items
**Validates: Requirements 4.5**

**Property 25: Mobile Sidebar Hidden**
*For any* viewport width < 768px, the sidebar SHALL be hidden by default
**Validates: Requirements 4.6**

### Search & Header Properties

**Property 26: Search Input Centering**
*For any* header, the search input SHALL be centered horizontally
**Validates: Requirements 5.2**

**Property 27: Search Input Width**
*For any* search input, the width SHALL be between 300px and 600px
**Validates: Requirements 5.3**

**Property 28: Search Debounce**
*For any* search input change, the search SHALL be debounced by 300ms
**Validates: Requirements 5.4**

**Property 29: Enter Key Search**
*For any* search input with text, pressing Enter SHALL immediately trigger search
**Validates: Requirements 5.5**

**Property 30: Clear Button Display**
*For any* search input with text, a clear button SHALL display
**Validates: Requirements 5.6**

### Form & Validation Properties

**Property 31: Real-time Validation**
*For any* form input, validation SHALL occur on input change
**Validates: Requirements 6.1**

**Property 32: Error Message Display**
*For any* invalid form field, an error message SHALL display below the field
**Validates: Requirements 6.2**

**Property 33: Save Validation**
*For any* settings save attempt, the save SHALL only proceed if all validations pass
**Validates: Requirements 6.3**

**Property 34: Success Notification**
*For any* successful settings save, a success notification SHALL display
**Validates: Requirements 6.4**

**Property 35: Checkbox Size**
*For any* checkbox control, the size SHALL be at least 16x16px
**Validates: Requirements 7.1**

**Property 36: Checkmark Contrast**
*For any* checked checkbox, the checkmark color SHALL contrast with the checkbox background by at least 4.5:1
**Validates: Requirements 7.3**

### Responsive Grid Properties

**Property 37: Grid Responsiveness**
*For any* movie grid, the column count SHALL adjust based on viewport width
**Validates: Requirements 8.1**

**Property 38: Grid Breakpoints**
*For any* viewport width < 640px, the grid SHALL display 2 columns
*For any* viewport width 640-768px, the grid SHALL display 3 columns
*For any* viewport width 768-1024px, the grid SHALL display 4 columns
*For any* viewport width 1024-1280px, the grid SHALL display 5 columns
*For any* viewport width > 1280px, the grid SHALL display 6 columns
**Validates: Requirements 8.2-8.6**

**Property 39: Grid Gap Spacing**
*For any* movie grid, the gap between tiles SHALL be 16px
**Validates: Requirements 8.7**

### Loading & Feedback Properties

**Property 40: Skeleton Placeholders**
*For any* loading state, skeleton placeholders matching the content layout SHALL display
**Validates: Requirements 9.1**

**Property 41: Toast Notifications**
*For any* download start or error, a toast notification SHALL display
**Validates: Requirements 9.3, 9.4**

**Property 42: Toast Auto-dismiss**
*For any* toast notification, auto-dismiss SHALL occur after 5 seconds
**Validates: Requirements 9.5**

### Localization Properties

**Property 43: Localization Load**
*For any* application startup, localization strings SHALL load
**Validates: Requirements 10.1**

**Property 44: Language Support**
*For any* language setting, English and Russian languages SHALL be available
**Validates: Requirements 10.2**

**Property 45: Missing Translation Fallback**
*For any* missing translation key, the key name SHALL display as fallback
**Validates: Requirements 10.3**

**Property 46: Language Persistence**
*For any* language change, the preference SHALL persist across sessions
**Validates: Requirements 10.4**

### Animation Properties

**Property 47: Page Navigation Animation**
*For any* page navigation, the transition animation SHALL complete in 200-300ms
**Validates: Requirements 11.1**

**Property 48: Hover Transitions**
*For any* interactive element hover, the transition SHALL complete in 150ms
**Validates: Requirements 11.2**

**Property 49: Tile Hover Scale**
*For any* movie tile hover, the scale transform SHALL be 1.05x
**Validates: Requirements 11.3**

**Property 50: Reduced Motion Respect**
*For any* user with prefers-reduced-motion enabled, animations SHALL be disabled
**Validates: Requirements 11.5**

### Accessibility Properties

**Property 51: ARIA Labels**
*For any* interactive element without visible text, an ARIA label SHALL be present
**Validates: Requirements 12.1**

**Property 52: Keyboard Navigation**
*For any* interactive feature, full keyboard navigation SHALL be supported
**Validates: Requirements 12.2**

**Property 53: Focus Indicators**
*For any* focusable element, a visible focus indicator SHALL be present
**Validates: Requirements 12.3**

**Property 54: Semantic HTML**
*For any* page structure, semantic HTML elements (nav, main, header, button) SHALL be used appropriately
**Validates: Requirements 12.4**

**Property 55: Modal Focus Trap**
*For any* open modal, focus SHALL be trapped within the modal until closed
**Validates: Requirements 12.5**

### Title Normalization Properties

**Property 56: Title Normalization**
*For any* movie title, release group tags, quality indicators, and encoding information SHALL be removed
**Validates: Requirements 13.1**

**Property 57: Title Capitalization**
*For any* English movie title, proper title case capitalization SHALL be applied
**Validates: Requirements 13.2**

**Property 58: Series Format**
*For any* TV series, season and episode numbers SHALL be formatted as S01E01
**Validates: Requirements 13.5**

### Quality Selection Properties

**Property 59: Quality Sorting**
*For any* quality list, options SHALL be sorted by resolution in descending order
**Validates: Requirements 14.3**

**Property 60: Recommended Quality**
*For any* quality list, the recommended option based on seeders and file size SHALL be highlighted
**Validates: Requirements 14.5**

### Pagination Properties

**Property 61: Infinite Scroll Auto-load**
*For any* infinite scroll mode, scrolling to 200px from bottom SHALL trigger auto-load
**Validates: Requirements 15.2**

**Property 62: Load More Button**
*For any* button pagination mode, a "Load More" button SHALL display at grid bottom
**Validates: Requirements 15.3**

**Property 63: Pagination Preference Persistence**
*For any* pagination mode change, the preference SHALL persist across sessions
**Validates: Requirements 15.7**

### State Persistence Properties

**Property 64: Feed Caching**
*For any* feed load, results SHALL be cached in memory
**Validates: Requirements 18.1**

**Property 65: Cache Persistence**
*For any* navigation away and return to Discover, the cached feed SHALL display without re-fetching
**Validates: Requirements 18.2**

**Property 66: Scroll Position Preservation**
*For any* return to cached view, the scroll position SHALL be restored
**Validates: Requirements 18.4**

**Property 67: Cache Invalidation**
*For any* cache, invalidation SHALL occur after 10 minutes of inactivity
**Validates: Requirements 18.6**

### Keyboard Shortcuts Properties

**Property 68: Ctrl+K Search Focus**
*For any* Ctrl/Cmd+K press, the search input SHALL receive focus
**Validates: Requirements 19.1**

**Property 69: Escape Close Modal**
*For any* open modal, pressing Escape SHALL close the modal
**Validates: Requirements 19.2**

**Property 70: Arrow Key Navigation**
*For any* movie grid, arrow keys SHALL navigate between tiles
**Validates: Requirements 19.3**

### Theme Properties

**Property 71: Theme Persistence**
*For any* theme change, the preference SHALL persist across sessions
**Validates: Requirements 20.3**

**Property 72: Theme Change Immediate**
*For any* theme change, the UI SHALL update immediately without reload
**Validates: Requirements 20.4**

### Modern Effects Properties

**Property 73: Header Glassmorphism**
*For any* header, a backdrop blur effect with semi-transparent background SHALL be applied
**Validates: Requirements 24.1**

**Property 74: Modal Backdrop Blur**
*For any* modal overlay, a backdrop blur effect SHALL be applied
**Validates: Requirements 24.3**

## Error Handling

### API Errors
- Display user-friendly error messages in toast notifications
- Provide retry buttons for failed operations
- Log detailed errors to console for debugging

### Validation Errors
- Display field-specific error messages below inputs
- Prevent form submission if validation fails
- Clear errors when user corrects input

### Network Errors
- Show connection error state
- Provide manual refresh option
- Cache data for offline viewing when possible

## Testing Strategy

### Unit Tests
- Test individual component rendering and props
- Test form validation logic
- Test store mutations and state management
- Test utility functions (title normalization, contrast calculation)

### Property-Based Tests
- **Property 1-74**: Each correctness property SHALL have a corresponding property-based test
- Use fast-check or similar library for generating test cases
- Run minimum 100 iterations per property test
- Tag each test with format: `**Feature: ui-redesign, Property X: [property text]**`

### Integration Tests
- Test page navigation and routing
- Test modal open/close flows
- Test search and filtering
- Test settings save/load

### Accessibility Tests
- Verify WCAG AA compliance for color contrast
- Test keyboard navigation paths
- Verify ARIA labels and semantic HTML
- Test screen reader compatibility

### Performance Tests
- Monitor bundle size
- Test grid rendering with 100+ tiles
- Test modal open/close performance
- Verify animations run at 60fps

## Deployment Considerations

- Update TailwindCSS from v3 to v4
- Update Svelte from v4 to v5 (optional, can be phased)
- Ensure backward compatibility with existing Tauri commands
- Test on multiple screen sizes and browsers
- Verify all keyboard shortcuts work as expected
