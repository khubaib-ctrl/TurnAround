use std::path::Path;
use crate::db::schema::{self, Commit, FileSnapshot};
use crate::vcs::object_store::ObjectStore;
use rusqlite::Connection;
use uuid::Uuid;
use chrono::Utc;

const PROJECT_EXTENSIONS: &[&str] = &[
    "prproj", "drp", "fcpxml", "otio", "xml", "edl", "aaf", "sesx", "als", "flp", "ptx",
];

const MEDIA_EXTENSIONS: &[&str] = &[
    // Video
    "mp4", "mov", "avi", "mkv", "mxf", "webm", "wmv", "flv", "m4v", "mpg", "mpeg", "ts", "r3d", "braw", "ari",
    // Audio
    "wav", "mp3", "aac", "flac", "ogg", "m4a", "aiff", "aif", "wma",
    // Images
    "png", "jpg", "jpeg", "tif", "tiff", "exr", "dpx", "bmp", "gif", "webp", "psd", "psb", "svg",
    // Subtitles / data
    "srt", "ass", "lut", "cube",
];

const FULL_COPY_SIZE_LIMIT: u64 = 50 * 1024 * 1024; // 50 MB

fn is_project_file(ext: &str) -> bool {
    PROJECT_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

fn is_tracked_extension(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    PROJECT_EXTENSIONS.contains(&lower.as_str()) || MEDIA_EXTENSIONS.contains(&lower.as_str())
}

pub fn create_commit(
    conn: &Connection,
    project_id: &str,
    project_root: &Path,
    message: &str,
    is_milestone: bool,
    object_store: &ObjectStore,
) -> Result<Commit, super::VcsError> {
    let branch = schema::get_active_branch(conn, project_id)?
        .ok_or(super::VcsError::NoActiveBranch)?;

    let changed_files = scan_tracked_files(project_root)?;
    if changed_files.is_empty() {
        return Err(super::VcsError::NoChanges);
    }

    let commit_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let commit = Commit {
        id: commit_id.clone(),
        project_id: project_id.to_string(),
        branch_id: branch.id.clone(),
        parent_id: branch.head_commit_id.clone(),
        message: message.to_string(),
        is_milestone,
        created_at: now,
    };

    schema::insert_commit(conn, &commit)?;

    for file_path in &changed_files {
        let abs_path = project_root.join(file_path);
        let metadata = std::fs::metadata(&abs_path)?;
        let file_size = metadata.len() as i64;

        let ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        let should_full_copy = is_project_file(&ext) || (file_size as u64) <= FULL_COPY_SIZE_LIMIT;

        let content_hash = if should_full_copy {
            let (hash, _) = object_store.store_file(&abs_path, conn)?;
            hash
        } else {
            crate::hasher::hash_file(&abs_path)?
        };

        let file_type = classify_file_type(&ext);

        let snapshot = FileSnapshot {
            id: Uuid::new_v4().to_string(),
            commit_id: commit_id.clone(),
            file_path: file_path.clone(),
            content_hash,
            file_size,
            file_type,
        };
        schema::insert_file_snapshot(conn, &snapshot)?;
    }

    schema::update_branch_head(conn, &branch.id, &commit_id)?;

    Ok(commit)
}

fn classify_file_type(ext: &str) -> String {
    let lower = ext.to_lowercase();
    match lower.as_str() {
        "mp4" | "mov" | "avi" | "mkv" | "mxf" | "webm" | "wmv" | "flv" | "m4v" | "mpg" | "mpeg" | "ts" | "r3d" | "braw" | "ari"
            => "video".to_string(),
        "wav" | "mp3" | "aac" | "flac" | "ogg" | "m4a" | "aiff" | "aif" | "wma"
            => "audio".to_string(),
        "png" | "jpg" | "jpeg" | "tif" | "tiff" | "exr" | "dpx" | "bmp" | "gif" | "webp" | "psd" | "psb" | "svg"
            => "image".to_string(),
        "prproj" | "drp" | "fcpxml" | "otio" | "xml" | "edl" | "aaf" | "sesx" | "als" | "flp" | "ptx"
            => "project".to_string(),
        "srt" | "ass" => "subtitle".to_string(),
        "lut" | "cube" => "lut".to_string(),
        _ => lower,
    }
}

fn scan_tracked_files(root: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut tracked = Vec::new();
    scan_dir_recursive(root, root, &mut tracked)?;
    Ok(tracked)
}

fn scan_dir_recursive(root: &Path, dir: &Path, results: &mut Vec<String>) -> Result<(), std::io::Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) {
            continue;
        }

        if path.is_dir() {
            scan_dir_recursive(root, &path, results)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if is_tracked_extension(ext) {
                if let Ok(rel) = path.strip_prefix(root) {
                    results.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(())
}

pub fn delete_commit(
    conn: &Connection,
    project_id: &str,
    commit_id: &str,
    object_store: &ObjectStore,
) -> Result<(), super::VcsError> {
    let commit = schema::get_commit(conn, commit_id)?
        .ok_or_else(|| super::VcsError::CommitNotFound(commit_id.to_string()))?;

    let branch = schema::get_active_branch(conn, project_id)?
        .ok_or(super::VcsError::NoActiveBranch)?;

    if branch.head_commit_id.as_deref() != Some(commit_id) {
        return Err(super::VcsError::CannotDeleteNonHeadCommit);
    }

    let hashes = schema::get_content_hashes_for_commit(conn, commit_id)?;

    for hash in &hashes {
        if schema::get_object(conn, hash)?.is_some() {
            let _ = object_store.remove_ref(hash, conn);
        }
    }

    schema::delete_commit(conn, commit_id)?;
    schema::update_branch_head(conn, &branch.id, commit.parent_id.as_deref().unwrap_or(""))?;

    if commit.parent_id.is_none() {
        conn.execute(
            "UPDATE branches SET head_commit_id = NULL WHERE id = ?1",
            rusqlite::params![branch.id],
        ).map_err(crate::db::DbError::from)?;
    }

    Ok(())
}

pub fn restore_commit(
    conn: &Connection,
    commit_id: &str,
    project_root: &Path,
    object_store: &ObjectStore,
) -> Result<RestoreReport, super::VcsError> {
    let _commit = schema::get_commit(conn, commit_id)?
        .ok_or_else(|| super::VcsError::CommitNotFound(commit_id.to_string()))?;

    let snapshots = schema::get_snapshots_for_commit(conn, commit_id)?;

    let mut restored: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for snapshot in &snapshots {
        let dest = project_root.join(&snapshot.file_path);

        let obj = schema::get_object(conn, &snapshot.content_hash)?;
        if let Some(_obj) = obj {
            let source = object_store.retrieve_path(&snapshot.content_hash);
            if source.exists() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&source, &dest)?;
                restored.push(snapshot.file_path.clone());
            } else {
                skipped.push(snapshot.file_path.clone());
            }
        } else {
            skipped.push(snapshot.file_path.clone());
        }
    }

    Ok(RestoreReport {
        total: snapshots.len(),
        restored_count: restored.len(),
        skipped_count: skipped.len(),
        restored,
        skipped,
    })
}

#[derive(serde::Serialize)]
pub struct RestoreReport {
    pub total: usize,
    pub restored_count: usize,
    pub skipped_count: usize,
    pub restored: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn export_commit(
    conn: &Connection,
    commit_id: &str,
    dest_dir: &Path,
    object_store: &ObjectStore,
) -> Result<ExportReport, super::VcsError> {
    let commit = schema::get_commit(conn, commit_id)?
        .ok_or_else(|| super::VcsError::CommitNotFound(commit_id.to_string()))?;

    let snapshots = schema::get_snapshots_for_commit(conn, commit_id)?;

    std::fs::create_dir_all(dest_dir)?;

    let mut exported: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for snapshot in &snapshots {
        let dest = dest_dir.join(&snapshot.file_path);

        let obj = schema::get_object(conn, &snapshot.content_hash)?;
        if let Some(_obj) = obj {
            let source = object_store.retrieve_path(&snapshot.content_hash);
            if source.exists() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&source, &dest)?;
                exported.push(snapshot.file_path.clone());
            } else {
                skipped.push(snapshot.file_path.clone());
            }
        } else {
            skipped.push(snapshot.file_path.clone());
        }
    }

    Ok(ExportReport {
        commit_message: commit.message,
        dest_path: dest_dir.to_string_lossy().to_string(),
        total: snapshots.len(),
        exported_count: exported.len(),
        skipped_count: skipped.len(),
        exported,
        skipped,
    })
}

#[derive(serde::Serialize)]
pub struct ExportReport {
    pub commit_message: String,
    pub dest_path: String,
    pub total: usize,
    pub exported_count: usize,
    pub skipped_count: usize,
    pub exported: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn get_detail(conn: &Connection, commit_id: &str) -> Result<(Commit, Vec<FileSnapshot>), super::VcsError> {
    let commit = schema::get_commit(conn, commit_id)?
        .ok_or_else(|| super::VcsError::CommitNotFound(commit_id.to_string()))?;
    let snapshots = schema::get_snapshots_for_commit(conn, commit_id)?;
    Ok((commit, snapshots))
}
