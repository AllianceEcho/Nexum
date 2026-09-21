//! Nexum media pipeline — probing, scheduling, muxing, and automation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Re-export existing types (backward compatible)
pub use {MediaType, Track, MediaProbe, MuxSpec};

/// Detected media subtype (e.g., mp4, mkv, mp3, png).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum MediaSubtype {
    Video,
    Audio,
    Image,
    Document,
    Other(String),
}

impl MediaSubtype {
    /// Infer subtype from a MIME type or extension string.
    pub fn from_mime(mime: &str) -> Self {
        match mime {
            m if m.starts_with("video/") => Self::Video,
            m if m.starts_with("audio/") => Self::Audio,
            m if m.starts_with("image/") => Self::Image,
            m if m.starts_with("application/pdf") || m.starts_with("text/") => Self::Document,
            m if m.starts_with("application/") || m.starts_with("x-") => Self::Other(m.to_owned()),
            _ => Self::Other(m.to_owned()),
        }
    }
}

impl std::fmt::Display for MediaSubtype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Video => write!(f, "video"),
            Self::Audio => write!(f, "audio"),
            Self::Image => write!(f, "image"),
            Self::Document => write!(f, "document"),
            Self::Other(name) => write!(f, "{name}"),
        }
    }
}

/// A segment of segmented media (HLS, DASH, etc.).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MediaSegment {
    pub index: u32,
    pub uri: String,
    pub duration: f64,
    pub size: Option<u64>,
    pub codecs: Vec<String>,
}

impl MediaSegment {
    pub fn new(index: u32, uri: impl Into<String>, duration: f64) -> Self {
        Self { index, uri: uri.into(), duration, size: None, codecs: Vec::new() }
    }
}

/// Manifest for segmented media (HLS, DASH).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct MediaManifest {
    pub playlist_uri: String,
    pub segments: Vec<MediaSegment>,
    pub codecs: Vec<String>,
    pub duration: Option<f64>,
    pub format: String,
}

impl MediaManifest {
    pub fn new(playlist_uri: impl Into<String>, format: impl Into<String>) -> Self {
        Self { playlist_uri: playlist_uri.into(), format: format.into(), ..Default::default() }
    }

    pub fn with_segments(mut self, segments: Vec<MediaSegment>) -> Self {
        self.segments = segments;
        self
    }

    /// Returns true if manifest has enough segments to proceed.
    pub fn is_complete(&self) -> bool {
        !self.segments.is_empty()
    }
}

/// Selection criteria for choosing which tracks to process.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct TrackSelection {
    pub prefer_codec: Option<String>,
    pub prefer_language: Option<String>,
    pub max_bitrate: Option<u64>,
    pub prefer_audio: bool,
    pub prefer_video: bool,
}

impl TrackSelection {
    pub fn new() -> Self { Self::default() }

    pub fn prefer_codec(mut self, codec: impl Into<String>) -> Self {
        self.prefer_codec = Some(codec.into());
        self
    }

    pub fn prefer_language(mut self, language: impl Into<String>) -> Self {
        self.prefer_language = Some(language.into());
        self
    }

    pub fn max_bitrate(mut self, max: u64) -> Self {
        self.max_bitrate = Some(max);
        self
    }

    pub fn prefer_audio(mut self) -> Self {
        self.prefer_audio = true;
        self
    }

    pub fn prefer_video(mut self) -> Self {
        self.prefer_video = true;
        self
    }

    /// Score a track against the selection criteria (higher is better).
    pub fn score(&self, track: &Track) -> u64 {
        let mut score: u64 = 0;
        if let Some(ref prefer_codec) = self.prefer_codec {
            if track.codec.to_lowercase().contains(prefer_codec) {
                score += 100;
            }
        }
        if let Some(ref prefer_language) = self.prefer_language {
            if track.language.as_deref() == Some(prefer_language.as_str()) {
                score += 100;
            }
        }
        if let Some(max_bitrate) = self.max_bitrate {
            if let Some(bitrate) = track.bitrate {
                if bitrate <= max_bitrate {
                    score += 50;
                }
            }
        }
        if self.prefer_video && track.index < 2 {
            score += 25;
        }
        if self.prefer_audio && track.index >= 2 {
            score += 25;
        }
        score
    }

    /// Select best track from a list (returns the highest scored).
    pub fn select(&self, tracks: &[Track]) -> Option<&Track> {
        let scored: Vec<(u64, &Track)> = tracks.iter().map(|t| (self.score(t), t)).collect();
        scored.into_iter().max_by_key(|&(score, _)| score).and_then(|(_, t)| if t.index < 100 { Some(t) } else { None })
    }
}

/// Scheduling policy for media processing segments.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum SchedulePolicy {
    Sequential,
    Parallel { max_concurrent: u32 },
    Adaptive,
}

impl Default for SchedulePolicy {
    fn default() -> Self { Self::Sequential }
}

impl SchedulePolicy {
    pub fn is_sequential(&self) -> bool { matches!(self, Self::Sequential) }
    pub fn max_concurrent(&self) -> Option<u32> {
        match self { Self::Parallel { max_concurrent } => Some(*max_concurrent), _ => None }
    }
}

/// An automated media processing pipeline step.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PipelineStep {
    pub id: u32,
    pub name: String,
    pub input: PathBuf,
    pub output: PathBuf,
    pub parameters: HashMap<String, String>,
    pub requires: Vec<u32>, // step IDs that must complete before this
}

impl PipelineStep {
    pub fn new(id: u32, name: impl Into<String>, input: PathBuf, output: PathBuf) -> Self {
        Self { id, name: name.into(), input, output, parameters: HashMap::new(), requires: Vec::new() }
    }

    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    pub fn requires_step(mut self, step_id: u32) -> Self {
        self.requires.push(step_id);
        self
    }
}

/// Media pipeline orchestration.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct MediaPipeline {
    pub input: PathBuf,
    pub output: PathBuf,
    pub format: String,
    pub steps: Vec<PipelineStep>,
    pub schedule: SchedulePolicy,
    pub metadata: HashMap<String, String>,
}

impl MediaPipeline {
    pub fn new(input: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self { input: input.into(), output: output.into(), format: "mp4".to_owned(), steps: Vec::new(), schedule: SchedulePolicy::default(), metadata: HashMap::new() }
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    pub fn with_step(mut self, step: PipelineStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn with_schedule(mut self, schedule: SchedulePolicy) -> Self {
        self.schedule = schedule;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Returns pipeline steps ordered by dependencies (topological sort).
    pub fn ordered_steps(&self) -> Option<Vec<&PipelineStep>> {
        let mut order = Vec::with_capacity(self.steps.len());
        let mut remaining: Vec<u32> = (0..self.steps.len() as u32).collect();
        let mut visited = Vec::new();

        while !remaining.is_empty() {
            let mut made_progress = false;
            let mut next_remaining = Vec::new();

            for step_id in &remaining {
                let step = self.steps.get(*step_id as usize)?;
                if step.requires.iter().all(|r| visited.contains(r)) {
                    order.push(step);
                    visited.push(*step_id);
                    made_progress = true;
                } else {
                    next_remaining.push(*step_id);
                }
            }

            if !made_progress {
                return None; // cycle detected
            }

            remaining = next_remaining;
        }

        Some(order)
    }

    /// Returns true if pipeline has all required steps.
    pub fn is_complete(&self) -> bool {
        !self.steps.is_empty() && self.ordered_steps().is_some()
    }
}

/// AI-based media analysis result.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct MediaAnalysis {
    pub file_path: PathBuf,
    pub mime_type: Option<String>,
    pub quality_score: f64,
    pub content_rating: Option<String>,
    pub recommended_tracks: Vec<u32>,
    pub recommended_format: Option<String>,
    pub summary: String,
}

impl MediaAnalysis {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self { file_path: file_path.into(), ..Default::default() }
    }

    pub fn with_mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    pub fn with_quality(mut self, score: f64) -> Self {
        self.quality_score = score;
        self
    }

    pub fn with_rating(mut self, rating: impl Into<String>) -> Self {
        self.content_rating = Some(rating.into());
        self
    }

    pub fn with_tracks(mut self, tracks: Vec<u32>) -> Self {
        self.recommended_tracks = tracks;
        self
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.recommended_format = Some(format.into());
        self
    }

    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }
}

/// MCP (Model Context Protocol) integration for AI-driven media automation.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct McpMediaRequest {
    pub file_path: PathBuf,
    pub action: String,
    pub parameters: HashMap<String, String>,
}

impl McpMediaRequest {
    pub fn new(file_path: impl Into<PathBuf>, action: impl Into<String>) -> Self {
        Self { file_path: file_path.into(), action: action.into(), parameters: HashMap::new() }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
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

impl std::fmt::Display for MediaManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "manifest: {} ({} segments)", self.playlist_uri, self.segments.len())
    }
}

impl std::fmt::Display for MediaPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pipeline: {} -> {} ({} steps)", self.input.display(), self.output.display(), self.steps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtype_from_mime() {
        assert_eq!(MediaSubtype::from_mime("video/mp4"), MediaSubtype::Video);
        assert_eq!(MediaSubtype::from_mime("audio/mpeg"), MediaSubtype::Audio);
        assert_eq!(MediaSubtype::from_mime("image/png"), MediaSubtype::Image);
        assert_eq!(MediaSubtype::from_mime("application/pdf"), MediaSubtype::Document);
    }

    #[test]
    fn media_segment_serializes() {
        let segment = MediaSegment::new(0, "/seg.m3u8", 10.0);
        let json = serde_json::to_string(&segment).unwrap();
        assert!(json.contains("\"index\":0"));
        assert!(json.contains("\"uri\":\"/seg.m3u8\""));
    }

    #[test]
    fn manifest_completeness() {
        let empty = MediaManifest::new("playlist.m3u8", "hls");
        assert!(!empty.is_complete());
        let with_segments = MediaManifest::new("playlist.m3u8", "hls")
            .with_segments(vec![MediaSegment::new(0, "/seg0.m3u8", 10.0)]);
        assert!(with_segments.is_complete());
    }

    #[test]
    fn track_selection_scores_tracks() {
        let tracks = vec![
            Track { id: 1, index: 0, codec: "h264".to_owned(), bitrate: Some(5000), language: None },
            Track { id: 2, index: 1, codec: "aac".to_owned(), bitrate: Some(128), language: Some("en".to_owned()) },
        ];
        let selection = TrackSelection::new()
            .prefer_codec("h264")
            .prefer_language("en");

        assert_eq!(selection.score(&tracks[0]), 100);
        assert_eq!(selection.score(&tracks[1]), 100);
        assert!(selection.select(&tracks).is_some());
    }

    #[test]
    fn pipeline_ordered_steps_topological() {
        let pipeline = MediaPipeline::new("/in.mp4", "/out.mkv")
            .with_step(PipelineStep::new(0, "probe", PathBuf::from("/in.mp4"), PathBuf::from("/probe.json")))
            .with_step(PipelineStep::new(1, "transcode", PathBuf::from("/in.mp4"), PathBuf::from("/out.mkv")).requires_step(0))
            .with_step(PipelineStep::new(2, "mux", PathBuf::from("/out.mkv"), PathBuf::from("/final.mkv")).requires_step(1));

        let ordered = pipeline.ordered_steps().unwrap();
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].name, "probe");
        assert_eq!(ordered[1].name, "transcode");
        assert_eq!(ordered[2].name, "mux");
    }

    #[test]
    fn pipeline_with_cycle_returns_none() {
        let pipeline = MediaPipeline::new("/in.mp4", "/out.mkv")
            .with_step(PipelineStep::new(0, "a", PathBuf::from("/in.mp4"), PathBuf::from("/b")).requires_step(1))
            .with_step(PipelineStep::new(1, "b", PathBuf::from("/in.mp4"), PathBuf::from("/out.mkv")).requires_step(0));

        assert!(pipeline.ordered_steps().is_none());
    }

    #[test]
    fn pipeline_completeness() {
        let empty = MediaPipeline::new("/in.mp4", "/out.mkv");
        assert!(!empty.is_complete());
        let with_step = MediaPipeline::new("/in.mp4", "/out.mkv")
            .with_step(PipelineStep::new(0, "probe", PathBuf::from("/in.mp4"), PathBuf::from("/out.json")));
        assert!(with_step.is_complete());
    }

    #[test]
    fn media_analysis_serializes() {
        let analysis = MediaAnalysis::new("/test.mp4")
            .with_mime_type("video/mp4")
            .with_quality(8.5)
            .with_rating("PG-13")
            .with_tracks(vec![1, 2])
            .with_format("mkv")
            .with_summary("High quality video");
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("\"file_path\":\"/test.mp4\""));
        assert!(json.contains("\"quality_score\":8.5"));
        assert!(json.contains("\"summary\":\"High quality video\""));
    }

    #[test]
    fn mcp_media_request() {
        let request = McpMediaRequest::new("/test.mp4", "analyze")
            .with_param("codec", "h264");
        assert_eq!(request.action, "analyze");
        assert!(request.parameters.contains_key("codec"));
    }
}
