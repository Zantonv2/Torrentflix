use regex::Regex;
use std::fmt;

/// Represents a file size with units
#[derive(Debug, Clone, PartialEq)]
pub struct Size {
    pub value: f64,
    pub unit: String,
}

impl Size {
    pub fn new(value: f64, unit: impl Into<String>) -> Self {
        Self {
            value,
            unit: unit.into(),
        }
    }
    
    /// Convert size to bytes
    pub fn to_bytes(&self) -> f64 {
        match self.unit.to_uppercase().as_str() {
            "B" | "BYTE" | "BYTES" => self.value,
            "KB" | "KILOBYTE" | "KILOBYTES" => self.value * 1024.0,
            "MB" | "MEGABYTE" | "MEGABYTES" => self.value * 1024.0 * 1024.0,
            "GB" | "GIGABYTE" | "GIGABYTES" => self.value * 1024.0 * 1024.0 * 1024.0,
            "TB" | "TERABYTE" | "TERABYTES" => self.value * 1024.0 * 1024.0 * 1024.0 * 1024.0,
            "PB" | "PETABYTE" | "PETABYTES" => self.value * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
            _ => self.value,
        }
    }
    
    /// Convert size to megabytes
    pub fn to_mb(&self) -> f64 {
        self.to_bytes() / (1024.0 * 1024.0)
    }
    
    /// Convert size to gigabytes
    pub fn to_gb(&self) -> f64 {
        self.to_bytes() / (1024.0 * 1024.0 * 1024.0)
    }
    
    /// Parse size from string
    pub fn parse(text: &str) -> Option<Self> {
        lazy_static::lazy_static! {
            static ref SIZE_RE: Regex = Regex::new(r"(?i)^(\d+\.?\d*)\s*([KMGT]?B)$").unwrap();
        }
        
        if let Some(captures) = SIZE_RE.captures(text.trim()) {
            let value = captures.get(1)?.as_str().parse::<f64>().ok()?;
            let unit = captures.get(2)?.as_str().to_uppercase();
            Some(Size::new(value, unit))
        } else {
            None
        }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit)
    }
}

/// Represents a frame rate
#[derive(Debug, Clone, PartialEq)]
pub struct FrameRate {
    pub value: f64,
}

impl FrameRate {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
    
    /// Parse frame rate from string
    pub fn parse(text: &str) -> Option<Self> {
        lazy_static::lazy_static! {
            static ref FRAMERATE_RE: Regex = Regex::new(r"(?i)^(\d+\.?\d*)\s*(FPS|FRAMES?\/SEC)?$").unwrap();
        }
        
        if let Some(captures) = FRAMERATE_RE.captures(text.trim()) {
            let value = captures.get(1)?.as_str().parse::<f64>().ok()?;
            Some(FrameRate::new(value))
        } else {
            None
        }
    }
    
    /// Check if this is a common frame rate
    pub fn is_common(&self) -> bool {
        const COMMON_RATES: &[f64] = &[23.976, 24.0, 25.0, 29.97, 30.0, 50.0, 59.94, 60.0, 120.0, 240.0];
        COMMON_RATES.iter().any(|&rate| (self.value - rate).abs() < 0.01)
    }
    
    /// Get the standard frame rate name
    pub fn get_standard_name(&self) -> Option<&'static str> {
        match self.value {
            v if (v - 23.976).abs() < 0.01 => Some("23.976 fps"),
            v if (v - 24.0).abs() < 0.01 => Some("24 fps"),
            v if (v - 25.0).abs() < 0.01 => Some("25 fps"),
            v if (v - 29.97).abs() < 0.01 => Some("29.97 fps"),
            v if (v - 30.0).abs() < 0.01 => Some("30 fps"),
            v if (v - 50.0).abs() < 0.01 => Some("50 fps"),
            v if (v - 59.94).abs() < 0.01 => Some("59.94 fps"),
            v if (v - 60.0).abs() < 0.01 => Some("60 fps"),
            v if (v - 120.0).abs() < 0.01 => Some("120 fps"),
            v if (v - 240.0).abs() < 0.01 => Some("240 fps"),
            _ => None,
        }
    }
}

impl fmt::Display for FrameRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.get_standard_name() {
            write!(f, "{}", name)
        } else {
            write!(f, "{:.2} fps", self.value)
        }
    }
}

/// Represents a bit rate
#[derive(Debug, Clone, PartialEq)]
pub struct BitRate {
    pub value: f64,
    pub unit: String,
}

impl BitRate {
    pub fn new(value: f64, unit: impl Into<String>) -> Self {
        Self {
            value,
            unit: unit.into(),
        }
    }
    
    /// Convert bit rate to bits per second
    pub fn to_bps(&self) -> f64 {
        match self.unit.to_uppercase().as_str() {
            "BPS" | "BIT/S" | "BITS/S" => self.value,
            "KBPS" | "KBIT/S" | "KBITS/S" => self.value * 1000.0,
            "MBPS" | "MBIT/S" | "MBITS/S" => self.value * 1000.0 * 1000.0,
            "GBPS" | "GBIT/S" | "GBITS/S" => self.value * 1000.0 * 1000.0 * 1000.0,
            _ => self.value,
        }
    }
    
    /// Parse bit rate from string
    pub fn parse(text: &str) -> Option<Self> {
        lazy_static::lazy_static! {
            static ref BITRATE_RE: Regex = Regex::new(r"(?i)^(\d+\.?\d*)\s*(K|M|G)?(BPS|BIT/S|BITS/S)$").unwrap();
        }
        
        if let Some(captures) = BITRATE_RE.captures(text.trim()) {
            let value = captures.get(1)?.as_str().parse::<f64>().ok()?;
            let prefix = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let unit = captures.get(3)?.as_str();
            let full_unit = format!("{}{}", prefix, unit);
            Some(BitRate::new(value, full_unit))
        } else {
            None
        }
    }
}

impl fmt::Display for BitRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} {}", self.value, self.unit)
    }
}

/// Represents a duration
#[derive(Debug, Clone, PartialEq)]
pub struct Duration {
    pub seconds: f64,
}

impl Duration {
    pub fn new(seconds: f64) -> Self {
        Self { seconds }
    }
    
    /// Create duration from minutes
    pub fn from_minutes(minutes: f64) -> Self {
        Self::new(minutes * 60.0)
    }
    
    /// Create duration from hours
    pub fn from_hours(hours: f64) -> Self {
        Self::new(hours * 3600.0)
    }
    
    /// Parse duration from string (supports various formats)
    pub fn parse(text: &str) -> Option<Self> {
        lazy_static::lazy_static! {
            static ref DURATION_RE: Regex = Regex::new(r"(?i)^(\d+\.?\d*)\s*(S|SEC|SECONDS?|M|MIN|MINUTES?|H|HR|HOURS?)?$").unwrap();
        }
        
        if let Some(captures) = DURATION_RE.captures(text.trim()) {
            let value = captures.get(1)?.as_str().parse::<f64>().ok()?;
            let unit = captures.get(2).map(|m| m.as_str().to_uppercase()).unwrap_or_default();
            
            let seconds = match unit.as_str() {
                "S" | "SEC" | "SECOND" | "SECONDS" => value,
                "M" | "MIN" | "MINUTE" | "MINUTES" => value * 60.0,
                "H" | "HR" | "HOUR" | "HOURS" => value * 3600.0,
                _ => value, // Default to seconds if no unit
            };
            
            Some(Duration::new(seconds))
        } else {
            None
        }
    }
    
    /// Get duration in human-readable format
    pub fn format(&self) -> String {
        if self.seconds < 60.0 {
            format!("{:.0}s", self.seconds)
        } else if self.seconds < 3600.0 {
            let minutes = (self.seconds / 60.0).round();
            format!("{:.0}m", minutes)
        } else {
            let hours = (self.seconds / 3600.0).round();
            format!("{:.0}h", hours)
        }
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Parse a quantity value from text, detecting the type automatically
pub fn parse_quantity(text: &str) -> Option<Quantity> {
    if let Some(size) = Size::parse(text) {
        Some(Quantity::Size(size))
    } else if let Some(framerate) = FrameRate::parse(text) {
        Some(Quantity::FrameRate(framerate))
    } else if let Some(bitrate) = BitRate::parse(text) {
        Some(Quantity::BitRate(bitrate))
    } else if let Some(duration) = Duration::parse(text) {
        Some(Quantity::Duration(duration))
    } else {
        None
    }
}

/// Enum representing different types of quantities
#[derive(Debug, Clone, PartialEq)]
pub enum Quantity {
    Size(Size),
    FrameRate(FrameRate),
    BitRate(BitRate),
    Duration(Duration),
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quantity::Size(size) => write!(f, "{}", size),
            Quantity::FrameRate(framerate) => write!(f, "{}", framerate),
            Quantity::BitRate(bitrate) => write!(f, "{}", bitrate),
            Quantity::Duration(duration) => write!(f, "{}", duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_size_parsing() {
        assert_eq!(Size::parse("1.5GB"), Some(Size::new(1.5, "GB")));
        assert_eq!(Size::parse("750MB"), Some(Size::new(750.0, "MB")));
        assert_eq!(Size::parse("2TB"), Some(Size::new(2.0, "TB")));
        assert_eq!(Size::parse("invalid"), None);
    }
    
    #[test]
    fn test_size_conversion() {
        let size_gb = Size::new(1.5, "GB");
        assert_eq!(size_gb.to_bytes(), 1.5 * 1024.0 * 1024.0 * 1024.0);
        assert_eq!(size_gb.to_mb(), 1.5 * 1024.0);
        assert_eq!(size_gb.to_gb(), 1.5);
        
        let size_mb = Size::new(1536.0, "MB");
        assert_eq!(size_mb.to_gb(), 1.5);
    }
    
    #[test]
    fn test_framerate_parsing() {
        assert_eq!(FrameRate::parse("23.976"), Some(FrameRate::new(23.976)));
        assert_eq!(FrameRate::parse("30 fps"), Some(FrameRate::new(30.0)));
        assert_eq!(FrameRate::parse("60FPS"), Some(FrameRate::new(60.0)));
        assert_eq!(FrameRate::parse("invalid"), None);
    }
    
    #[test]
    fn test_framerate_common() {
        assert!(FrameRate::new(24.0).is_common());
        assert!(FrameRate::new(30.0).is_common());
        assert!(FrameRate::new(60.0).is_common());
        assert!(!FrameRate::new(45.0).is_common());
    }
    
    #[test]
    fn test_framerate_standard_name() {
        assert_eq!(FrameRate::new(24.0).get_standard_name(), Some("24 fps"));
        assert_eq!(FrameRate::new(23.976).get_standard_name(), Some("23.976 fps"));
        assert_eq!(FrameRate::new(45.0).get_standard_name(), None);
    }
    
    #[test]
    fn test_bitrate_parsing() {
        assert_eq!(BitRate::parse("320kbps"), Some(BitRate::new(320.0, "kbps")));
        assert_eq!(BitRate::parse("5.1 Mbps"), Some(BitRate::new(5.1, "Mbps")));
        assert_eq!(BitRate::parse("1000 bps"), Some(BitRate::new(1000.0, "bps")));
        assert_eq!(BitRate::parse("invalid"), None);
    }
    
    #[test]
    fn test_bitrate_conversion() {
        let bitrate_kbps = BitRate::new(320.0, "kbps");
        assert_eq!(bitrate_kbps.to_bps(), 320000.0);
        
        let bitrate_mbps = BitRate::new(5.0, "Mbps");
        assert_eq!(bitrate_mbps.to_bps(), 5000000.0);
    }
    
    #[test]
    fn test_duration_parsing() {
        assert_eq!(Duration::parse("120s"), Some(Duration::new(120.0)));
        assert_eq!(Duration::parse("5 min"), Some(Duration::new(300.0)));
        assert_eq!(Duration::parse("2 hours"), Some(Duration::new(7200.0)));
        assert_eq!(Duration::parse("invalid"), None);
    }
    
    #[test]
    fn test_duration_formatting() {
        assert_eq!(Duration::new(45.0).format(), "45s");
        assert_eq!(Duration::new(120.0).format(), "2m");
        assert_eq!(Duration::new(3600.0).format(), "1h");
    }
    
    #[test]
    fn test_parse_quantity() {
        assert!(matches!(parse_quantity("1.5GB"), Some(Quantity::Size(_))));
        assert!(matches!(parse_quantity("30 fps"), Some(Quantity::FrameRate(_))));
        assert!(matches!(parse_quantity("320kbps"), Some(Quantity::BitRate(_))));
        assert!(matches!(parse_quantity("5 min"), Some(Quantity::Duration(_))));
        assert_eq!(parse_quantity("invalid"), None);
    }
    
    #[test]
    fn test_quantity_display() {
        assert_eq!(format!("{}", parse_quantity("1.5GB").unwrap()), "1.50 GB");
        assert_eq!(format!("{}", parse_quantity("30 fps").unwrap()), "30 fps");
    }
}
