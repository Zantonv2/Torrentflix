// TypeScript types for Netflix-style UI
// These match the Rust DTOs from engine/src/ui/dto.rs

export interface UiSearchResult {
  id: string;
  title: string;
  year?: number;
  poster_url?: string;
  backdrop_url?: string;
  quality_badge: string;
  rating?: number;
  runtime_minutes?: number;
  category?: string; // Movie genre/category
  description?: string; // Movie description/overview
  cast?: string[]; // Cast members
  genres?: string[]; // Movie genres
  torrent_info: UiTorrentInfo;
}

export interface UiTorrentInfo {
  seeders: number;
  size_gb: number;
  magnet_link: string;
}

export interface UiSearchRequest {
  query: string;
  media_type: string;
  limit?: number;
}

export interface UiSearchResponse {
  results: UiSearchResult[];
  total: number;
  took_ms: number;
}

// Component props
export interface MovieCardProps {
  movie: UiSearchResult;
}

export interface SearchBarProps {
  onSearch: (query: string) => void;
}

export interface MovieGridProps {
  movies: UiSearchResult[];
}

// UI Events
export interface SearchEvent {
  detail: string;
}
