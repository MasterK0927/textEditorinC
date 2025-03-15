use crate::core::{EditorError, FileManager, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileSystem {
    current_directory: PathBuf,
}

impl FileSystem {
    pub fn new() -> Result<Self> {
        let current_directory = std::env::current_dir()
            .map_err(|e| EditorError::Io(e))?;

        Ok(Self {
            current_directory,
        })
    }

    pub fn get_current_directory(&self) -> &Path {
        &self.current_directory
    }

    pub fn set_current_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if path.is_dir() {
            self.current_directory = path.to_path_buf();
            Ok(())
        } else {
            Err(EditorError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Directory not found",
            )))
        }
    }

    pub fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.current_directory.join(path)
        }
    }

    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        self.resolve_path(path).exists()
    }

    pub fn is_readable<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = self.resolve_path(path);
        path.exists() && path.is_file()
    }

    pub fn is_writable<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = self.resolve_path(path);

        if path.exists() {
            // Check if we can write to existing file
            let metadata = fs::metadata(&path)?;
            Ok(!metadata.permissions().readonly())
        } else {
            // Check if we can create file in directory
            if let Some(parent) = path.parent() {
                Ok(parent.exists() && parent.is_dir())
            } else {
                Ok(false)
            }
        }
    }

    pub fn backup_file<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = self.resolve_path(path);
        let backup_path = path.with_extension(
            format!("{}.backup", path.extension().unwrap_or_default().to_string_lossy())
        );

        if path.exists() {
            fs::copy(&path, &backup_path)?;
        }

        Ok(backup_path)
    }

    pub fn get_file_metadata<P: AsRef<Path>>(&self, path: P) -> Result<FileMetadata> {
        let path = self.resolve_path(path);
        let metadata = fs::metadata(&path)?;

        Ok(FileMetadata {
            size: metadata.len(),
            readonly: metadata.permissions().readonly(),
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        })
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            current_directory: PathBuf::from("."),
        })
    }
}

impl FileManager for FileSystem {
    fn open(&self, filename: &str) -> Result<String> {
        let path = self.resolve_path(filename);

        if !path.exists() {
            return Err(EditorError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", filename),
            )));
        }

        if !self.is_readable(&path) {
            return Err(EditorError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Cannot read file: {}", filename),
            )));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| EditorError::Io(e))?;

        Ok(content)
    }

    fn save(&self, filename: &str, content: &str) -> Result<()> {
        let path = self.resolve_path(filename);

        // Create backup if file exists
        if path.exists() {
            self.backup_file(&path)?;
        }

        // Check if we can write to the file
        if !self.is_writable(&path)? {
            return Err(EditorError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Cannot write to file: {}", filename),
            )));
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(&path, content)
            .map_err(|e| EditorError::Io(e))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub readonly: bool,
    pub modified: Option<std::time::SystemTime>,
    pub created: Option<std::time::SystemTime>,
}

pub struct SafeFileManager {
    file_system: FileSystem,
    auto_backup: bool,
    max_file_size: u64,
}

impl SafeFileManager {
    pub fn new(auto_backup: bool, max_file_size: u64) -> Result<Self> {
        Ok(Self {
            file_system: FileSystem::new()?,
            auto_backup,
            max_file_size,
        })
    }

    pub fn file_system(&self) -> &FileSystem {
        &self.file_system
    }

    pub fn set_auto_backup(&mut self, enabled: bool) {
        self.auto_backup = enabled;
    }

    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }

    fn validate_file_size(&self, content: &str) -> Result<()> {
        if content.len() as u64 > self.max_file_size {
            return Err(EditorError::InvalidOperation(
                format!("File size exceeds maximum limit of {} bytes", self.max_file_size)
            ));
        }
        Ok(())
    }

    fn validate_filename(filename: &str) -> Result<()> {
        if filename.is_empty() {
            return Err(EditorError::InvalidOperation("Filename cannot be empty".to_string()));
        }

        if filename.contains('\0') {
            return Err(EditorError::InvalidOperation("Filename cannot contain null bytes".to_string()));
        }

        // Check for invalid characters on Windows
        if cfg!(windows) {
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            if filename.chars().any(|c| invalid_chars.contains(&c)) {
                return Err(EditorError::InvalidOperation("Filename contains invalid characters".to_string()));
            }
        }

        Ok(())
    }
}

impl FileManager for SafeFileManager {
    fn open(&self, filename: &str) -> Result<String> {
        Self::validate_filename(filename)?;

        let content = self.file_system.open(filename)?;

        // Check file size
        if content.len() as u64 > self.max_file_size {
            return Err(EditorError::InvalidOperation(
                format!("File size exceeds maximum limit of {} bytes", self.max_file_size)
            ));
        }

        Ok(content)
    }

    fn save(&self, filename: &str, content: &str) -> Result<()> {
        Self::validate_filename(filename)?;
        self.validate_file_size(content)?;

        // Create automatic backup if enabled and file exists
        if self.auto_backup && self.file_system.file_exists(filename) {
            let backup_path = self.file_system.backup_file(filename)?;
            eprintln!("Backup created: {}", backup_path.display());
        }

        self.file_system.save(filename, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_system_operations() {
        let temp_dir = tempdir().unwrap();
        let mut fs = FileSystem::new().unwrap();
        fs.set_current_directory(temp_dir.path()).unwrap();

        let test_file = "test.txt";
        let test_content = "Hello, World!";

        // Test save
        assert!(fs.save(test_file, test_content).is_ok());

        // Test file exists
        assert!(fs.file_exists(test_file));

        // Test open
        let content = fs.open(test_file).unwrap();
        assert_eq!(content, test_content);

        // Test metadata
        let metadata = fs.get_file_metadata(test_file).unwrap();
        assert_eq!(metadata.size, test_content.len() as u64);
    }

    #[test]
    fn test_safe_file_manager() {
        let temp_dir = tempdir().unwrap();
        let mut safe_manager = SafeFileManager::new(true, 1024).unwrap();
        safe_manager.file_system.set_current_directory(temp_dir.path()).unwrap();

        let test_file = "safe_test.txt";
        let test_content = "Safe content";

        // Test save with validation
        assert!(safe_manager.save(test_file, test_content).is_ok());

        // Test open with validation
        let content = safe_manager.open(test_file).unwrap();
        assert_eq!(content, test_content);

        // Test file size limit
        let large_content = "x".repeat(2048);
        assert!(safe_manager.save("large.txt", &large_content).is_err());
    }

    #[test]
    fn test_filename_validation() {
        assert!(SafeFileManager::validate_filename("valid.txt").is_ok());
        assert!(SafeFileManager::validate_filename("").is_err());
        assert!(SafeFileManager::validate_filename("file\0name").is_err());
    }
}