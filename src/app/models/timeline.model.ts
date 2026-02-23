export interface RationalTime {
  value: number;
  rate: number;
}

export interface TimeRange {
  start: RationalTime;
  duration: RationalTime;
}

export type TrackKind = 'Video' | 'Audio';

export interface Clip {
  name: string;
  media_ref: string | null;
  source_range: TimeRange | null;
  trimmed_range: TimeRange | null;
}

export interface Track {
  name: string;
  kind: TrackKind;
  clips: Clip[];
}

export interface Timeline {
  name: string;
  tracks: Track[];
  duration: RationalTime | null;
}

export type DiffStatus = 'Added' | 'Removed' | 'Modified' | 'Unchanged';

export interface ClipDiff {
  name: string;
  status: DiffStatus;
  media_ref: string | null;
  old_range: TimeRange | null;
  new_range: TimeRange | null;
  track_index: number;
  clip_index: number;
}

export interface TrackDiff {
  name: string;
  kind: TrackKind;
  clips: ClipDiff[];
}

export interface DiffSummary {
  added: number;
  removed: number;
  modified: number;
  unchanged: number;
}

export interface TimelineDiff {
  old_name: string;
  new_name: string;
  tracks: TrackDiff[];
  summary: DiffSummary;
}
