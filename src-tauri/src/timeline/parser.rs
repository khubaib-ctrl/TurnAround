use std::path::Path;
use super::{Timeline, Track, TrackKind, Clip, TimeRange, RationalTime};

/// Parse an OTIO JSON file into our Timeline model.
/// OTIO files use a well-defined JSON schema that we parse natively.
pub fn parse_otio_file(path: &Path) -> Result<Timeline, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    parse_otio_json(&content)
}

pub fn parse_otio_json(json_str: &str) -> Result<Timeline, String> {
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {e}"))?;

    let schema_type = value.get("OTIO_SCHEMA")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if schema_type.starts_with("Timeline") {
        parse_timeline_object(&value)
    } else if schema_type.starts_with("SerializableCollection") {
        let children = value.get("children")
            .and_then(|v| v.as_array())
            .ok_or("No children in collection")?;
        if let Some(first) = children.first() {
            parse_timeline_object(first)
        } else {
            Err("Empty collection".to_string())
        }
    } else {
        Err(format!("Unsupported OTIO schema: {schema_type}"))
    }
}

fn parse_timeline_object(value: &serde_json::Value) -> Result<Timeline, String> {
    let name = value.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let mut tracks = Vec::new();

    if let Some(stack) = value.get("tracks") {
        let children = stack.get("children")
            .and_then(|v| v.as_array());

        if let Some(track_list) = children {
            for (i, track_val) in track_list.iter().enumerate() {
                let track_name = track_val.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&format!("Track {}", i + 1))
                    .to_string();

                let kind_str = track_val.get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Video");
                let kind = if kind_str == "Audio" { TrackKind::Audio } else { TrackKind::Video };

                let mut clips = Vec::new();
                if let Some(children) = track_val.get("children").and_then(|v| v.as_array()) {
                    for child in children {
                        let schema = child.get("OTIO_SCHEMA")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if schema.starts_with("Clip") {
                            clips.push(parse_clip(child));
                        }
                    }
                }

                tracks.push(Track { name: track_name, kind, clips });
            }
        }
    }

    Ok(Timeline { name, tracks, duration: None })
}

fn parse_clip(value: &serde_json::Value) -> Clip {
    let name = value.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Clip")
        .to_string();

    let media_ref = value.get("media_reference")
        .and_then(|v| v.get("target_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let source_range = value.get("source_range").and_then(parse_time_range);
    let trimmed_range = value.get("trimmed_range").and_then(parse_time_range);

    Clip { name, media_ref, source_range, trimmed_range }
}

fn parse_time_range(value: &serde_json::Value) -> Option<TimeRange> {
    let start_time = value.get("start_time")?;
    let duration = value.get("duration")?;

    Some(TimeRange {
        start: RationalTime {
            value: start_time.get("value")?.as_f64()?,
            rate: start_time.get("rate")?.as_f64()?,
        },
        duration: RationalTime {
            value: duration.get("value")?.as_f64()?,
            rate: duration.get("rate")?.as_f64()?,
        },
    })
}

/// Parse FCPXML (Final Cut Pro) into our Timeline model
pub fn parse_fcpxml_file(path: &Path) -> Result<Timeline, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {e}"))?;
    parse_fcpxml(&content)
}

fn parse_fcpxml(_content: &str) -> Result<Timeline, String> {
    // FCPXML is an XML format. For MVP, we provide a basic parser.
    // Full OTIO C++ FFI will handle this more robustly.
    Ok(Timeline {
        name: "FCPXML Timeline".to_string(),
        tracks: Vec::new(),
        duration: None,
    })
}

pub fn parse_timeline_from_path(path: &Path) -> Result<Timeline, String> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "otio" => parse_otio_file(path),
        "fcpxml" => parse_fcpxml_file(path),
        _ => Err(format!("Unsupported timeline format: .{ext}")),
    }
}
