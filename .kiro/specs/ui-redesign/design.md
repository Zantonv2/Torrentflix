# Design Document: TorrentFlix UI Redesign

## Overview

Complete UI redesign: 23 bloated components → 15 focused components. Full Russian localization. All 26+ backend commands wired.

---

## UI Architecture

### Layout Structure

```
┌─────────────────────────────────────────────────────┐
│  Header (60px)                                      │
│  Logo "TORRENTFLIX" | Search Input | [Search Btn]  │
├──────────────┬──────────────────────────────────────┤
│ Sidebar      │ Content Area                         │
│ (200px)      │                                      │
│              │ Discover / Library / Downloads /     │
│ • Открыть    │ Settings                             │
│ • Библиотека │                                      │
│ • Загрузки   │                                      │
│ • Настройки  │                                      │
└──────────────┴──────────────────────────────────────┘
```

**Mobile (<768px)**: Sidebar collapses to hamburger menu

---

## Components & UI Details

### Core Components (8)

#### 1. Button.svelte
**Variants**: primary (red #E50914), secondary (gray), ghost (transparent), danger (red)
**Sizes**: sm (8px padding), md (12px), lg (16px)
**States**: default, hover, disabled, loading (spinner)
**Usage**: All clickable actions

#### 2. Input.svelte
**Types**: text, password, number, email
**Features**: 
- Label above input
- Placeholder text
- Error message below (red text)
- Masked password toggle (eye icon)
- Focus border: #E50914
**Styling**: Dark gray background (#2a2a2a), white text, 8px border-radius

#### 3. Modal.svelte
**Structure**:
```
┌─────────────────────────────┐
│ Title              [X Close] │
├─────────────────────────────┤
│ Content (scrollable)        │
├─────────────────────────────┤
│ [Cancel] [Action Button]    │
└─────────────────────────────┘
```
**Backdrop**: Black 75% opacity
**Focus trap**: Tab cycles within modal only
**Escape key**: Closes modal

#### 4. Card.svelte
**Styling**: Dark background (#1a1a1a), border (#404040), 8px radius
**Hover**: Slight shadow, scale 1.02
**Padding**: 16px default
**Usage**: Movie cards, list items, settings sections

#### 5. Badge.svelte
**Variants**: 
- default (gray)
- success (green)
- warning (yellow)
- error (red)
- info (blue)
**Sizes**: sm (8px padding), md (12px)
**Usage**: Quality labels, status indicators

#### 6. Loading.svelte
**Types**:
- spinner: Rotating circle (Netflix red)
- skeleton: Gray placeholder boxes (shimmer animation)
- dots: Three bouncing dots

#### 7. Toast.svelte
**Position**: Bottom-right corner
**Auto-dismiss**: 3s (success), 5s (error)
**Types**: success (green), error (red), info (blue), warning (yellow)
**Content**: Icon + message text

#### 8. Tabs.svelte
**Style**: Underline tabs, active tab has red underline
**Content**: Switches on tab click
**Keyboard**: Arrow keys to navigate

---

### Layout Components (3)

#### 1. MainLayout.svelte
**Grid**: `display: flex; flex-direction: column; height: 100vh`
- Header: 60px fixed
- Main: flex-1, flex-direction: row
  - Sidebar: 200px (or 0 on mobile)
  - Content: flex-1, overflow-y: auto

#### 2. Header.svelte
**Content**:
- Left: Logo "TORRENTFLIX" (red, 24px font)
- Center: Search input (flex-1)
  - Placeholder: "Поиск фильмов..."
  - Debounce: 300ms
  - On Enter: Call `invoke('search_movies', {query, media_type: 'movie', limit: 50})`
- Right: Search button (red, 40px)

#### 3. Sidebar.svelte
**Navigation items** (4):
```
🎬 Открыть      (Discover)
📚 Библиотека   (Library)
📥 Загрузки     (Downloads)
⚙️ Настройки    (Settings)
```
**Active state**: Red background, white text
**Inactive**: Gray text, hover: light gray background
**Mobile**: Hamburger icon, slides in from left

---

### Page Components (4)

#### 1. Discover.svelte

**On Mount**: Call `invoke('get_feed')` → display results

**Layout**:
```
┌─────────────────────────────────────────┐
│ Search: [Input] [Search Button]         │
├─────────────────────────────────────────┤
│ Movie Grid (responsive columns)         │
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│ │ 🎬   │ │ 🎬   │ │ 🎬   │ │ 🎬   │   │
│ │ Title│ │ Title│ │ Title│ │ Title│   │
│ │ 2024 │ │ 2024 │ │ 2024 │ │ 2024 │   │
│ │ ⭐8.5│ │ ⭐8.5│ │ ⭐8.5│ │ ⭐8.5│   │
│ └──────┘ └──────┘ └──────┘ └──────┘   │
└─────────────────────────────────────────┘
```

**Movie Card**:
- Poster image (200x300px, 2:3 aspect ratio)
- Hover: Overlay with "View Details" button
- Title (truncate 2 lines)
- Year
- Rating (⭐ + number)
- Quality badge (top-right)

**Grid Columns**:
- <640px: 1 column
- 640-1024px: 2-3 columns
- >1024px: 4-6 columns

**Click card**: Open Modal with full details
- Title, description, genres, cast, runtime
- Ratings: Kinopoisk, IMDb, TMDB
- Download button (red)

**Search**: Call `invoke('search_movies', {query, media_type: 'movie', limit: 50})`

**Loading**: Show skeleton placeholders (gray boxes)

**Empty**: "Ничего не найдено" message

**Error**: Red error box + "Повторить" button

---

#### 2. Library.svelte

**Tabs** (4):
1. **Браузер** (Browser)
   - Search input: "Поиск в библиотеке..."
   - Filters: Year, Resolution, Rating (slider), Watch Status
   - Grid/List toggle
   - On mount: Call `invoke('query_library', {filters: {}})`
   - On search: Call `invoke('search_library', {query, options: {fuzzy: true}})`
   - Click item: Call `invoke('get_media_item', {id})` + `invoke('get_file_versions', {media_id})`
   - Show detail modal with metadata

2. **Коллекции** (Collections)
   - List of collections (left panel, 300px)
   - Collection items (right panel, flex-1)
   - "+ New" button: Open modal to create collection
     - Input: Name, Description
     - Call `invoke('create_collection', {name, description})`
   - "🧠 Smart" button: Open modal for smart collection
     - Inputs: Name, Title pattern, Min rating, Watch status
     - Call `invoke('create_smart_collection', {name, criteria})`
   - Click collection: Call `invoke('get_collection_items', {collection_id})`

3. **Аналитика** (Analytics)
   - On mount: Call `invoke('get_storage_analytics')`
   - Display:
     - Total size (GB)
     - Media item count
     - File version count
     - Avg versions per media
     - Breakdown by resolution (pie chart or bars)
     - Breakdown by quality
     - Breakdown by status
     - Largest items (top 10)
   - Duplicate space warning (yellow box)

4. **Очистка** (Cleanup)
   - On mount: Call `invoke('get_cleanup_candidates')`
   - List candidates with:
     - File name
     - Size (GB)
     - Reason (duplicate, low quality, etc.)
   - Checkbox to select candidates
   - "Очистить" button: Show confirmation modal
     - "Вы уверены? Это удалит X файлов (Y GB)"
     - [Cancel] [Удалить]
     - Call `invoke('execute_cleanup', {candidate_ids})`

---

#### 3. Downloads.svelte

**On Mount**: 
- Call `invoke('get_active_downloads')`
- Start polling every 5 seconds

**Layout**:
```
┌─────────────────────────────────────────┐
│ Active Downloads                        │
├─────────────────────────────────────────┤
│ ┌─────────────────────────────────────┐ │
│ │ Movie Title                         │ │
│ │ ████████░░░░░░░░░░░░░░░░░░░░░░░░░░ │ │
│ │ 45% | 2.5 GB / 5.5 GB | 1.2 MB/s  │ │
│ │ [Пауза] [Возобновить] [Удалить]   │ │
│ └─────────────────────────────────────┘ │
│ ┌─────────────────────────────────────┐ │
│ │ Another Movie                       │ │
│ │ ████████████░░░░░░░░░░░░░░░░░░░░░░ │ │
│ │ 72% | 3.9 GB / 5.5 GB | 0.8 MB/s  │ │
│ │ [Пауза] [Возобновить] [Удалить]   │ │
│ └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

**Download Item**:
- Title (bold)
- Progress bar (red fill, gray background)
- Stats: percentage, downloaded/total, speed
- Buttons:
  - Пауза: Call `invoke('pause_download', {hash})`
  - Возобновить: Call `invoke('resume_download', {hash})`
  - Удалить: Show confirmation → Call `invoke('delete_download', {hash, delete_files: false})`

**Empty State**: "Нет активных загрузок"

**Polling**: Update progress bars every 5 seconds

---

#### 4. Settings.svelte

**Sections** (scrollable):

1. **🔑 API & Metadata**
   - TMDB API Key (Input, masked)
   - Kinopoisk Token (Input, masked)

2. **🧲 qBittorrent**
   - Enable checkbox
   - Host (Input, default: localhost)
   - Port (Input, default: 5555)
   - Username (Input)
   - Password (Input, masked)
   - Use HTTPS (checkbox)
   - Verify SSL (checkbox)
   - "Проверить соединение" button
     - Call `invoke('test_qbittorrent', {url, username, password})`
     - Show success/error toast

3. **📁 Filesystem**
   - Library Path (Input + Browse button)
     - Browse: Call `invoke('pick_folder')` → update field
   - Download Path (Input + Browse button)
     - Browse: Call `invoke('pick_folder')` → update field

4. **🔔 Notifications**
   - Telegram Enable (checkbox)
     - Bot Token (Input, masked)
     - Chat ID (Input)
     - Debounce (Input, number)
   - Email Enable (checkbox)
     - SMTP Host (Input)
     - SMTP Port (Input)
     - Username (Input)
     - Password (Input, masked)
     - From Email (Input)
     - To Email (Input)

5. **🎛️ Application**
   - Log Level (Dropdown: DEBUG, INFO, WARN, ERROR)
   - Search Limit (Input, 1-1000)
   - Search Timeout (Input, 1-300 seconds)
   - Max Concurrent Downloads (Input, 1-100)
   - Enabled Indexers (Input, comma-separated)
   - Debug Mode (checkbox)

6. **🌐 Proxy**
   - Enable Proxy (checkbox)
   - Proxy Address (Input)
   - Proxy Port (Input)

**Bottom**: 
- "Сохранить" button (red)
  - Call `invoke('save_settings', {settings})`
  - Show success/error toast
  - Display validation errors inline (red text below field)

**Dirty state**: Track unsaved changes, warn on navigation

---

## Tauri Command Mapping

| Component | Action | Command | Parameters |
|-----------|--------|---------|------------|
| Discover | Load feed | `get_feed` | none |
| Discover | Search | `search_movies` | query, media_type, limit |
| Discover | Download | `start_download` | magnetLink, title |
| Library Browser | Load | `query_library` | filters |
| Library Browser | Search | `search_library` | query, options |
| Library Browser | View item | `get_media_item` | id |
| Library Browser | View versions | `get_file_versions` | media_id |
| Library Collections | Create | `create_collection` | name, description |
| Library Collections | Smart create | `create_smart_collection` | name, criteria |
| Library Collections | View items | `get_collection_items` | collection_id |
| Library Collections | Add items | `add_to_collection` | collection_id, media_ids |
| Library Analytics | Load | `get_storage_analytics` | none |
| Library Cleanup | Load candidates | `get_cleanup_candidates` | none |
| Library Cleanup | Execute | `execute_cleanup` | candidate_ids |
| Downloads | Load | `get_active_downloads` | none |
| Downloads | Pause | `pause_download` | hash |
| Downloads | Resume | `resume_download` | hash |
| Downloads | Delete | `delete_download` | hash, delete_files |
| Settings | Load | `get_settings` | none |
| Settings | Save | `save_settings` | settings |
| Settings | Test qBittorrent | `test_qbittorrent` | url, username, password |
| Settings | Pick folder | `pick_folder` | none |

---

## Styling

**Colors**:
- Primary Red: #E50914 (Netflix red)
- Dark BG: #0f0f0f
- Card BG: #1a1a1a
- Border: #404040
- Text: #ffffff
- Text Secondary: #999999
- Error: #ff4444
- Success: #44ff44
- Warning: #ffaa00

**Typography**:
- Logo: 24px bold
- Headings: 20px bold
- Body: 14px regular
- Small: 12px regular

**Spacing**: 8px grid (8, 16, 24, 32px)

**Responsive**:
- Mobile: <640px (1 column, hamburger nav)
- Tablet: 640-1024px (2-3 columns)
- Desktop: >1024px (4-6 columns)

---

## Russian Localization

All text in Russian via svelte-i18n:
- Navigation labels
- Button text
- Placeholder text
- Error messages
- Empty states
- Loading messages
- Validation errors
- Toast messages

---

## State Management

**5 Svelte Stores**:
1. `searchStore`: query, results, loading, error
2. `libraryStore`: items, filters, selectedItem, collections, loading, error
3. `downloadStore`: active, loading, error, pollInterval
4. `settingsStore`: data, dirty, loading, saving, errors
5. `uiStore`: currentPage, modal, toast, sidebarCollapsed

All stores update atomically on command success/failure.
