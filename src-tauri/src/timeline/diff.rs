use serde::{Deserialize, Serialize};
use super::{Timeline, Track, Clip, TrackKind, TimeRange};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffStatus {
    Added,
    Removed,
    Modified,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipDiff {
    pub name: String,
    pub status: DiffStatus,
    pub media_ref: Option<String>,
    pub old_range: Option<TimeRange>,
    pub new_range: Option<TimeRange>,
    pub track_index: usize,
    pub clip_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDiff {
    pub name: String,
    pub kind: TrackKind,
    pub clips: Vec<ClipDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineDiff {
    pub old_name: String,
    pub new_name: String,
    pub tracks: Vec<TrackDiff>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
}

pub fn diff_timelines(old: &Timeline, new: &Timeline) -> TimelineDiff {
    let max_tracks = old.tracks.len().max(new.tracks.len());
    let mut tracks = Vec::new();
    let mut summary = DiffSummary { added: 0, removed: 0, modified: 0, unchanged: 0 };

    for i in 0..max_tracks {
        let old_track = old.tracks.get(i);
        let new_track = new.tracks.get(i);

        match (old_track, new_track) {
            (Some(ot), Some(nt)) => {
                let track_diff = diff_tracks(ot, nt, i, &mut summary);
                tracks.push(track_diff);
            }
            (Some(ot), None) => {
                let clips: Vec<ClipDiff> = ot.clips.iter().enumerate().map(|(ci, c)| {
                    summary.removed += 1;
                    ClipDiff {
                        name: c.name.clone(),
                        status: DiffStatus::Removed,
                        media_ref: c.media_ref.clone(),
                        old_range: c.source_range.clone(),
                        new_range: None,
                        track_index: i,
                        clip_index: ci,
                    }
                }).collect();
                tracks.push(TrackDiff { name: ot.name.clone(), kind: ot.kind.clone(), clips });
            }
            (None, Some(nt)) => {
                let clips: Vec<ClipDiff> = nt.clips.iter().enumerate().map(|(ci, c)| {
                    summary.added += 1;
                    ClipDiff {
                        name: c.name.clone(),
                        status: DiffStatus::Added,
                        media_ref: c.media_ref.clone(),
                        old_range: None,
                        new_range: c.source_range.clone(),
                        track_index: i,
                        clip_index: ci,
                    }
                }).collect();
                tracks.push(TrackDiff { name: nt.name.clone(), kind: nt.kind.clone(), clips });
            }
            (None, None) => {}
        }
    }

    TimelineDiff {
        old_name: old.name.clone(),
        new_name: new.name.clone(),
        tracks,
        summary,
    }
}

fn diff_tracks(old: &Track, new: &Track, track_idx: usize, summary: &mut DiffSummary) -> TrackDiff {
    let mut clip_diffs = Vec::new();

    let mut matched_new: Vec<bool> = vec![false; new.clips.len()];

    for (oi, old_clip) in old.clips.iter().enumerate() {
        let mut found = false;
        for (ni, new_clip) in new.clips.iter().enumerate() {
            if matched_new[ni] {
                continue;
            }
            if clips_match(old_clip, new_clip) {
                matched_new[ni] = true;
                found = true;

                if clips_identical(old_clip, new_clip) {
                    summary.unchanged += 1;
                    clip_diffs.push(ClipDiff {
                        name: new_clip.name.clone(),
                        status: DiffStatus::Unchanged,
                        media_ref: new_clip.media_ref.clone(),
                        old_range: old_clip.source_range.clone(),
                        new_range: new_clip.source_range.clone(),
                        track_index: track_idx,
                        clip_index: ni,
                    });
                } else {
                    summary.modified += 1;
                    clip_diffs.push(ClipDiff {
                        name: new_clip.name.clone(),
                        status: DiffStatus::Modified,
                        media_ref: new_clip.media_ref.clone(),
                        old_range: old_clip.source_range.clone(),
                        new_range: new_clip.source_range.clone(),
                        track_index: track_idx,
                        clip_index: ni,
                    });
                }
                break;
            }
        }

        if !found {
            summary.removed += 1;
            clip_diffs.push(ClipDiff {
                name: old_clip.name.clone(),
                status: DiffStatus::Removed,
                media_ref: old_clip.media_ref.clone(),
                old_range: old_clip.source_range.clone(),
                new_range: None,
                track_index: track_idx,
                clip_index: oi,
            });
        }
    }

    for (ni, new_clip) in new.clips.iter().enumerate() {
        if !matched_new[ni] {
            summary.added += 1;
            clip_diffs.push(ClipDiff {
                name: new_clip.name.clone(),
                status: DiffStatus::Added,
                media_ref: new_clip.media_ref.clone(),
                old_range: None,
                new_range: new_clip.source_range.clone(),
                track_index: track_idx,
                clip_index: ni,
            });
        }
    }

    TrackDiff {
        name: new.name.clone(),
        kind: new.kind.clone(),
        clips: clip_diffs,
    }
}

fn clips_match(a: &Clip, b: &Clip) -> bool {
    match (&a.media_ref, &b.media_ref) {
        (Some(ref_a), Some(ref_b)) => ref_a == ref_b,
        _ => a.name == b.name,
    }
}

fn clips_identical(a: &Clip, b: &Clip) -> bool {
    if !clips_match(a, b) {
        return false;
    }
    ranges_equal(&a.source_range, &b.source_range)
}

fn ranges_equal(a: &Option<TimeRange>, b: &Option<TimeRange>) -> bool {
    match (a, b) {
        (Some(ra), Some(rb)) => {
            (ra.start.value - rb.start.value).abs() < 0.001
                && (ra.start.rate - rb.start.rate).abs() < 0.001
                && (ra.duration.value - rb.duration.value).abs() < 0.001
                && (ra.duration.rate - rb.duration.rate).abs() < 0.001
        }
        (None, None) => true,
        _ => false,
    }
}
