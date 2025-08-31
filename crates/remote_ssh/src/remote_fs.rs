//! Remote File System Operations
//!
//! Provides file system operations over SSH using SFTP, integrated with Zed's FS abstraction.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Simple file type enumeration
#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}

/// Simple directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub path: PathBuf,
    pub file_type: FileType,
}

/// Simple file metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    pub size: u64,
    pub is_dir: bool,
    pub is_file: bool,
    pub modified: std::time::SystemTime,
}

/// Remote file system implementation
pub struct RemoteFs {
    /// SSH connection
    connection: Arc<Mutex<super::ssh_connection::SshConnection>>,
    /// Remote working directory
    working_dir: Mutex<PathBuf>,
}

impl RemoteFs {
    /// Create a new remote file system
    pub fn new(
        connection: Arc<Mutex<super::ssh_connection::SshConnection>>,
    ) -> Self {
        Self {
            connection,
            working_dir: Mutex::new(PathBuf::from("/")),
        }
    }

    /// Get the current working directory
    pub fn working_directory(&self) -> PathBuf {
        self.working_dir.lock().unwrap().clone()
    }

    /// Set the working directory
    pub fn set_working_directory(&self, path: PathBuf) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let metadata = sftp.stat(&path)
            .with_context(|| format!("Failed to stat path: {}", path.display()))?;

        if !metadata.is_dir() {
            anyhow::bail!("Path is not a directory: {}", path.display());
        }

        *self.working_dir.lock().unwrap() = path;
        Ok(())
    }

    /// Resolve a path relative to the working directory
    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_directory().join(path)
        }
    }

    /// List files in a directory
    pub fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_path = self.resolve_path(path);

        let entries = sftp.readdir(&resolved_path)
            .with_context(|| format!("Failed to read directory: {}", resolved_path.display()))?;

        let mut result = Vec::new();
        for (path, stat) in entries {
            let file_type = if stat.is_dir() {
                FileType::Directory
            } else if stat.is_file() {
                FileType::File
            } else {
                FileType::Symlink
            };

            result.push(DirEntry {
                path,
                file_type,
            });
        }

        Ok(result)
    }

    /// Get file metadata
    pub fn metadata(&self, path: &Path) -> Result<Metadata> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_path = self.resolve_path(path);

        let stat = sftp.stat(&resolved_path)
            .with_context(|| format!("Failed to stat file: {}", resolved_path.display()))?;

        Ok(Metadata {
            size: stat.size.unwrap_or(0),
            is_dir: stat.is_dir(),
            is_file: stat.is_file(),
            modified: std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(stat.mtime.unwrap_or(0) as u64),
        })
    }

    /// Read a file's contents
    pub fn load(&self, path: &Path) -> Result<String> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_path = self.resolve_path(path);

        let mut file = sftp.open(&resolved_path)
            .with_context(|| format!("Failed to open file: {}", resolved_path.display()))?;

        let mut contents = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut contents)?;

        String::from_utf8(contents)
            .context("File contains invalid UTF-8")
    }

    /// Write data to a file
    pub fn save(&self, path: &Path, content: &str) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_path = self.resolve_path(path);

        let mut file = sftp.create(&resolved_path)
            .with_context(|| format!("Failed to create file: {}", resolved_path.display()))?;

        use std::io::Write;
        file.write_all(content.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Check if a path exists
    pub fn is_file(&self, path: &Path) -> bool {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            return false;
        }

        let sftp = conn.sftp();
        if let Ok(sftp) = sftp {
            let resolved_path = self.resolve_path(path);
            sftp.stat(&resolved_path).is_ok()
        } else {
            false
        }
    }

    /// Create a directory
    pub fn create_dir(&self, path: &Path) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_path = self.resolve_path(path);

        sftp.mkdir(&resolved_path, 0o755)
            .with_context(|| format!("Failed to create directory: {}", resolved_path.display()))?;

        Ok(())
    }

    /// Remove a file or directory
    pub fn remove(&self, path: &Path) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_path = self.resolve_path(path);

        let metadata = sftp.stat(&resolved_path)
            .with_context(|| format!("Failed to stat path: {}", resolved_path.display()))?;

        if metadata.is_dir() {
            sftp.rmdir(&resolved_path)
                .with_context(|| format!("Failed to remove directory: {}", resolved_path.display()))?;
        } else {
            sftp.unlink(&resolved_path)
                .with_context(|| format!("Failed to remove file: {}", resolved_path.display()))?;
        }

        Ok(())
    }

    /// Rename/move a file or directory
    pub fn rename(&self, old_path: &Path, new_path: &Path) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        if !conn.is_connected() {
            anyhow::bail!("Not connected to remote server");
        }

        let sftp = conn.sftp()?;
        let resolved_old = self.resolve_path(old_path);
        let resolved_new = self.resolve_path(new_path);

        sftp.rename(&resolved_old, &resolved_new, None)
            .with_context(|| format!("Failed to rename {} to {}", resolved_old.display(), resolved_new.display()))?;

        Ok(())
    }

}

/// Remote file system wrapper for basic operations
pub struct RemoteFsWrapper {
    inner: Arc<RemoteFs>,
}

impl RemoteFsWrapper {
    pub fn new(remote_fs: Arc<RemoteFs>) -> Self {
        Self { inner: remote_fs }
    }

    /// Load a file from the remote system
    pub fn load(&self, path: &Path) -> Result<String> {
        self.inner.load(path)
    }

    /// Save a file to the remote system
    pub fn save(&self, path: &Path, content: &str) -> Result<()> {
        self.inner.save(path, content)
    }

    /// Check if path is a file
    pub fn is_file(&self, path: &Path) -> bool {
        self.inner.is_file(path)
    }

    /// Create a directory
    pub fn create_dir(&self, path: &Path) -> Result<()> {
        self.inner.create_dir(path)
    }

    /// Remove a file or directory
    pub fn remove(&self, path: &Path) -> Result<()> {
        self.inner.remove(path)
    }

    /// Rename/move a file or directory
    pub fn rename(&self, old_path: &Path, new_path: &Path) -> Result<()> {
        self.inner.rename(old_path, new_path)
    }

    /// List directory contents
    pub fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        self.inner.read_dir(path)
    }

    /// Get file metadata
    pub fn metadata(&self, path: &Path) -> Result<Metadata> {
        self.inner.metadata(path)
    }
}