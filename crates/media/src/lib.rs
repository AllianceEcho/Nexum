//! Nexum media pipeline — types for probing and inspecting media files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Detected media type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum MediaType {
    Audio,
    Video,
    Image,
    Document,
    Unknown,
}

impl MediaType {
    /// Infer media type from a MIME type string.
    pub fn from_mime(mime: &str) -> Self {
        match mime {
            m if m.starts_with("audio/") => Self::Audio,
            m if m.starts_with("video/") => Self::Video,
            m if m.starts_with("image/") => Self::Image,
            m if m.starts_with("application/pdf") || m.starts_with("text/") => Self::Document,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio => write!(f, "audio"),
            Self::Video => write!(f, "video"),
            Self::Image => write!(f, "image"),
            Self::Document => write!(f, "document"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// A track within a media file (audio channel, video stream, subtitle, etc.).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Track {
    pub id: u32,
    pub index: u32,
    pub codec: String,
    pub bitrate: Option<u64>,
    pub language: Option<String>,
}

/// Probe result for a media file.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct MediaProbe {
    pub file_path: PathBuf,
    pub mime_type: Option<String>,
    pub media_type: Option<MediaType>,
    pub size: Option<u64>,
    pub duration: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub tracks: Vec<Track>,
}

impl MediaProbe {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self { file_path: file_path.into(), ..Default::default() }
    }

    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self.media_type = self.media_type.or(Some(MediaType::from_mime(self.mime_type.as_deref().unwrap_or("unknown"))));
        self
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_duration(mut self, duration: u64) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_tracks(mut self, tracks: Vec<Track>) -> Self {
        self.tracks = tracks;
        self
    }

    /// Determine media type from the MIME type (if set), falling back to Unknown.
    pub fn infer_media_type(&mut self) {
        if let Some(ref mime) = self.mime_type {
            self.media_type = Some(MediaType::from_mime(mime));
        }
    }

    /// Returns true if this probe has enough data to proceed with processing.
    pub fn is_complete(&self) -> bool {
        self.mime_type.is_some() && self.size.is_some()
    }
}

impl std::fmt::Display for MediaProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_path.display())?;
        if let Some(ref mime) = self.mime_type {
            write!(f, " ({mime})")?;
        }
        if let Some(size) = self.size {
            write!(f, " {size} bytes")?;
        }
        Ok(())
    }
}

/// A muxer for combining media tracks into a single output file.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MuxSpec {
    pub output_path: PathBuf,
    pub format: String,
    pub tracks: Vec<u32>, // track IDs to include
}

impl MuxSpec {
    pub fn new(output_path: impl Into<PathBuf>, format: impl Into<String>) -> Self {
        Self { output_path: output_path.into(), format: format.into(), tracks: Vec::new() }
    }

    pub fn with_track(mut self, track_id: u32) -> Self {
        self.tracks.push(track_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_media_type_from_mime() {
        assert_eq!(MediaType::from_mime("audio/mpeg"), MediaType::Audio);
        assert_eq!(MediaType::from_mime("video/mp4"), MediaType::Video);
        assert_eq!(MediaType::from_mime("image/png"), MediaType::Image);
        assert_eq!(MediaType::from_mime("application/pdf"), MediaType::Document);
        assert_eq!(MediaType::from_mime("application/octet-stream"), MediaType::Unknown);
    }

    #[test]
    fn media_probe_display() {
        let probe = MediaProbe::new("/tmp/test.mp4")
            .with_mime_type("video/mp4")
            .with_size(1024);
        let display = format!("{}", probe);
        assert!(display.contains("test.mp4"));
        assert!(display.contains("video/mp4"));
        assert!(display.contains("1024 bytes"));
    }

    #[test]
    fn media_probe_completeness() {
        let empty = MediaProbe::new("/tmp/test");
        assert!(!empty.is_complete());
        let with_mime = MediaProbe::new("/tmp/test").with_mime_type("video/mp4");
        assert!(!with_mime.is_complete()); // still no size
        let complete = MediaProbe::new("/tmp/test").with_mime_type("video/mp4").with_size(100);
        assert!(complete.is_complete());
    }

    #[test]
    fn media_probe_sets_media_type_from_mime() {
        let probe = MediaProbe::new("/tmp/test").with_mime_type("audio/flac");
        assert_eq!(probe.media_type, Some(MediaType::Audio));
    }

    #[test]
    fn mux_spec_serializes() {
        let spec = MuxSpec::new("/out.mp4", "mp4").with_track(1).with_track(2);
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"tracks\":[1,2]"));
    }
}
