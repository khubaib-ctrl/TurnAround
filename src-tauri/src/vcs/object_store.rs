use std::fs;
use std::path::{Path, PathBuf};
use crate::db::schema::{self, StoredObject};
use crate::hasher;
use rusqlite::Connection;

pub struct ObjectStore {
    base_path: PathBuf,
}

impl ObjectStore {
    pub fn new(editgit_dir: &Path) -> Self {
        let base_path = editgit_dir.join("objects");
        Self { base_path }
    }

    pub fn init(&self) -> std::io::Result<()> {
        fs::create_dir_all(&self.base_path)
    }

    fn object_path(&self, hash: &str) -> PathBuf {
        let (prefix, rest) = hash.split_at(2);
        self.base_path.join(prefix).join(rest)
    }

    pub fn store_file(&self, source: &Path, conn: &Connection) -> Result<(String, i64), super::VcsError> {
        let hash = hasher::hash_file(source)?;
        let file_size = fs::metadata(source)?.len() as i64;

        if let Some(_existing) = schema::get_object(conn, &hash)? {
            schema::increment_object_ref(conn, &hash)?;
            return Ok((hash, file_size));
        }

        let dest = self.object_path(&hash);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, &dest)?;

        let obj = StoredObject {
            hash: hash.clone(),
            size: file_size,
            stored_path: dest.to_string_lossy().to_string(),
            ref_count: 1,
        };
        schema::insert_object(conn, &obj)?;

        Ok((hash, file_size))
    }

    pub fn retrieve_path(&self, hash: &str) -> PathBuf {
        self.object_path(hash)
    }

    pub fn remove_ref(&self, hash: &str, conn: &Connection) -> Result<(), super::VcsError> {
        let remaining = schema::decrement_object_ref(conn, hash)?;
        if remaining <= 0 {
            let path = self.object_path(hash);
            if path.exists() {
                fs::remove_file(&path)?;
            }
            schema::delete_object(conn, hash)?;
        }
        Ok(())
    }
}
