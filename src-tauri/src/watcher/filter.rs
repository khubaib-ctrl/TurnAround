use std::path::Path;

const PROJECT_EXTENSIONS: &[&str] = &[
    "prproj", "drp", "db", "fcpxml", "otio", "xml", "edl", "aaf", "sesx", "als", "flp", "ptx",
];

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "avi", "mkv", "mxf", "webm", "wmv", "flv", "m4v", "mpg", "mpeg", "ts", "r3d", "braw", "ari",
    "wav", "mp3", "aac", "flac", "ogg", "m4a", "aiff", "aif", "wma",
    "png", "jpg", "jpeg", "tif", "tiff", "exr", "dpx", "bmp", "gif", "webp", "psd", "psb", "svg",
    "srt", "ass", "lut", "cube",
];

pub fn is_tracked_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let lower = ext.to_lowercase();
        PROJECT_EXTENSIONS.contains(&lower.as_str()) || MEDIA_EXTENSIONS.contains(&lower.as_str())
    } else {
        false
    }
}
