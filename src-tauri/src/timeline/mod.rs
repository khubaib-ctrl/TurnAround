pub mod parser;
pub mod diff;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RationalTime {
    pub value: f64,
    pub rate: f64,
}

impl RationalTime {
    pub fn new(value: f64, rate: f64) -> Self {
        Self { value, rate }
    }

    pub fn seconds(&self) -> f64 {
        if self.rate == 0.0 { 0.0 } else { self.value / self.rate }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: RationalTime,
    pub duration: RationalTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub name: String,
    pub media_ref: Option<String>,
    pub source_range: Option<TimeRange>,
    pub trimmed_range: Option<TimeRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub kind: TrackKind,
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub name: String,
    pub tracks: Vec<Track>,
    pub duration: Option<RationalTime>,
}
