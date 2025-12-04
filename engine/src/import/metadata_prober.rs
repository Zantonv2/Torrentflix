// Metadata Prober Component
// Extracts technical metadata from media files using ffprobe

use anyhow::{Result, Context, bail};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use tracing::debug;

// Re-export library model types for metadata
pub use crate::library::models::{TechnicalMetadata, VideoInfo, AudioTrack, SubtitleTrack, Resolution};

// ============================================================================
// ffprobe JSON structures
// ============================================================================

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    format: FfprobeFormat,
    streams: Vec<FfprobeStream>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    format_name: String,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    index: u32,
    codec_type: String,
    codec_name: String,
    
    // Video fields
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    bit_rate: Option<String>,
    pix_fmt: Option<String>,
    color_space: Option<String>,
    
    // Audio fields
    channels: Option<u32>,
    sample_rate: Option<String>,
    
    // Common fields
    tags: Option<FfprobeTags>,
}

#[derive(Debug, Deserialize)]
struct FfprobeTags {
    language: Option<String>,
    title: Option<String>,
    #[serde(rename = "DURATION")]
    duration: Option<String>,
}

// ============================================================================
// MetadataProber
// ============================================================================

/// Component for probing media file metadata using ffprobe
pub struct MetadataProber {
    /// Path to ffprobe binary
    ffprobe_path: String,
}

impl MetadataProber {
    /// Create a new MetadataProber
    ///
    /// # Arguments
    /// * `ffprobe_path` - Optional path to ffprobe binary. If None, uses "ffprobe" from PATH
    pub fn new(ffprobe_path: Option<String>) -> Self {
        Self {
            ffprobe_path: ffprobe_path.unwrap_or_else(|| "ffprobe".to_string()),
        }
    }
    
    /// Probe a media file and extract complete technical metadata
    ///
    /// # Arguments
    /// * `path` - Path to the media file
    ///
    /// # Returns
    /// Complete technical metadata for the file
    ///
    /// # Requirements
    /// - Requirements: 2.2
    ///
    /// # Errors
    /// Returns an error if:
    /// - File does not exist
    /// - ffprobe is not available
    /// - ffprobe fails to parse the file
    /// - Output cannot be parsed
    pub async fn probe_file(&self, path: &Path) -> Result<TechnicalMetadata> {
        debug!("Probing file: {:?}", path);
        
        // Validate file exists
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }
        
        // Get file size
        let file_size = std::fs::metadata(path)
            .with_context(|| format!("Failed to read file metadata: {}", path.display()))?
            .len();
        
        // Run ffprobe
        let output = self.run_ffprobe(path).await?;
        
        // Parse output
        let ffprobe_output: FfprobeOutput = serde_json::from_str(&output)
            .context("Failed to parse ffprobe JSON output")?;
        
        // Extract metadata
        let metadata = self.extract_metadata(ffprobe_output, file_size)?;
        
        debug!("Probed file successfully: {:?}", path);
        Ok(metadata)
    }
    
    /// Run ffprobe and get JSON output
    async fn run_ffprobe(&self, path: &Path) -> Result<String> {
        let output = Command::new(&self.ffprobe_path)
            .arg("-v")
            .arg("quiet")
            .arg("-print_format")
            .arg("json")
            .arg("-show_format")
            .arg("-show_streams")
            .arg(path)
            .output()
            .with_context(|| format!("Failed to execute ffprobe (is it installed?)"))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("ffprobe failed: {}", stderr);
        }
        
        let stdout = String::from_utf8(output.stdout)
            .context("ffprobe output is not valid UTF-8")?;
        
        Ok(stdout)
    }
    
    /// Extract metadata from ffprobe output
    fn extract_metadata(&self, output: FfprobeOutput, file_size: u64) -> Result<TechnicalMetadata> {
        // Parse container format
        let container = output.format.format_name.clone();
        
        // Parse duration (convert to u64 seconds, rounding up)
        let duration = output.format.duration
            .as_ref()
            .and_then(|d| d.parse::<f64>().ok())
            .map(|d| d.ceil() as u64)
            .unwrap_or(0);
        
        // Parse bitrate
        let bitrate = output.format.bit_rate
            .as_ref()
            .and_then(|b| b.parse::<u64>().ok())
            .unwrap_or(0);
        
        // Extract video info
        let video = self.extract_video_info(&output.streams);
        
        // Extract audio tracks
        let audio_tracks = self.extract_audio_tracks(&output.streams);
        
        // Extract subtitle tracks
        let subtitle_tracks = self.extract_subtitle_tracks(&output.streams);
        
        Ok(TechnicalMetadata {
            container,
            video,
            audio_tracks,
            subtitle_tracks,
            duration,
            bitrate,
            file_size,
        })
    }
    
    /// Extract video information from streams
    fn extract_video_info(&self, streams: &[FfprobeStream]) -> Option<VideoInfo> {
        // Find first video stream
        let video_stream = streams.iter()
            .find(|s| s.codec_type == "video")?;
        
        let codec = video_stream.codec_name.clone();
        let width = video_stream.width?;
        let height = video_stream.height?;
        
        // Parse framerate (format: "24000/1001" or "24")
        let framerate = video_stream.r_frame_rate.as_ref()
            .and_then(|r| self.parse_framerate(r))
            .unwrap_or(0.0);
        
        // Parse bitrate
        let bitrate = video_stream.bit_rate.as_ref()
            .and_then(|b| b.parse::<u64>().ok())
            .unwrap_or(0);
        
        let color_space = video_stream.color_space.clone();
        
        Some(VideoInfo {
            codec,
            resolution: Resolution { width, height },
            framerate,
            bitrate,
            color_space,
        })
    }
    
    /// Parse framerate from ffprobe format (e.g., "24000/1001" or "24")
    fn parse_framerate(&self, framerate_str: &str) -> Option<f32> {
        if let Some(slash_pos) = framerate_str.find('/') {
            // Format: "numerator/denominator"
            let numerator = framerate_str[..slash_pos].parse::<f32>().ok()?;
            let denominator = framerate_str[slash_pos + 1..].parse::<f32>().ok()?;
            if denominator > 0.0 {
                Some(numerator / denominator)
            } else {
                None
            }
        } else {
            // Format: "24"
            framerate_str.parse::<f32>().ok()
        }
    }
    
    /// Extract audio tracks from streams
    fn extract_audio_tracks(&self, streams: &[FfprobeStream]) -> Vec<AudioTrack> {
        streams.iter()
            .filter(|s| s.codec_type == "audio")
            .filter_map(|s| {
                let codec = s.codec_name.clone();
                let channels = s.channels?;
                
                let bitrate = s.bit_rate.as_ref()
                    .and_then(|b| b.parse::<u64>().ok())
                    .unwrap_or(0);
                
                let language = s.tags.as_ref()
                    .and_then(|t| t.language.clone());
                
                Some(AudioTrack {
                    codec,
                    language,
                    channels,
                    bitrate,
                })
            })
            .collect()
    }
    
    /// Extract subtitle tracks from streams
    fn extract_subtitle_tracks(&self, streams: &[FfprobeStream]) -> Vec<SubtitleTrack> {
        streams.iter()
            .filter(|s| s.codec_type == "subtitle")
            .map(|s| {
                let codec = s.codec_name.clone();
                
                let language = s.tags.as_ref()
                    .and_then(|t| t.language.clone());
                
                SubtitleTrack {
                    language,
                    codec,
                }
            })
            .collect()
    }
    
    /// Extract video information only (lighter operation)
    pub async fn extract_video_info_only(&self, path: &Path) -> Result<Option<VideoInfo>> {
        let metadata = self.probe_file(path).await?;
        Ok(metadata.video)
    }
    
    /// Extract audio information only
    pub async fn extract_audio_info(&self, path: &Path) -> Result<Vec<AudioTrack>> {
        let metadata = self.probe_file(path).await?;
        Ok(metadata.audio_tracks)
    }
    
    /// Extract subtitle information only
    pub async fn extract_subtitle_info(&self, path: &Path) -> Result<Vec<SubtitleTrack>> {
        let metadata = self.probe_file(path).await?;
        Ok(metadata.subtitle_tracks)
    }
    
    /// Get duration only (lighter operation)
    pub async fn get_duration(&self, path: &Path) -> Result<u64> {
        let metadata = self.probe_file(path).await?;
        Ok(metadata.duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_framerate() {
        let prober = MetadataProber::new(None);
        
        // Test fraction format - use approximate comparison for floating point
        let result1 = prober.parse_framerate("24000/1001");
        assert!(result1.is_some());
        assert!((result1.unwrap() - 23.976).abs() < 0.001);
        
        let result2 = prober.parse_framerate("30000/1001");
        assert!(result2.is_some());
        assert!((result2.unwrap() - 29.970).abs() < 0.001);
        
        // Test simple format
        assert_eq!(prober.parse_framerate("24"), Some(24.0));
        assert_eq!(prober.parse_framerate("30"), Some(30.0));
        
        // Test invalid formats
        assert_eq!(prober.parse_framerate("invalid"), None);
        assert_eq!(prober.parse_framerate("24/0"), None);
    }
}
