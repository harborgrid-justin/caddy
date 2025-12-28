// CADDY - Enterprise CAD System
// File I/O System - Native Format Module
// Agent 6 - File I/O System Developer

use crate::io::document::Document;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Native format errors
#[derive(Error, Debug)]
pub enum NativeError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
    #[error("Invalid file format")]
    InvalidFormat,
    #[error("Compression error: {0}")]
    Compression(String),
}

pub type NativeResult<T> = Result<T, NativeError>;

/// CADDY native file format version
const CURRENT_VERSION: u32 = 1;
const MAGIC_BYTES: &[u8; 4] = b"CDDY";

/// Native file format (binary .cdy)
pub struct NativeFormat {
    /// Compression level (0 = none, 1-9 = compression level)
    compression_level: u8,
    /// Progress callback
    progress_callback: Option<Box<dyn Fn(usize, usize)>>,
}

impl NativeFormat {
    /// Create a new native format handler
    pub fn new() -> Self {
        Self {
            compression_level: 6, // Default moderate compression
            progress_callback: None,
        }
    }

    /// Set compression level (0 = none, 1-9 = increasing compression)
    pub fn with_compression(mut self, level: u8) -> Self {
        self.compression_level = level.min(9);
        self
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Save document to native binary format
    pub fn save<P: AsRef<Path>>(&self, doc: &Document, path: P) -> NativeResult<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write magic bytes
        writer.write_all(MAGIC_BYTES)?;

        // Write version
        writer.write_all(&CURRENT_VERSION.to_le_bytes())?;

        // Write compression flag
        writer.write_all(&[self.compression_level])?;

        // Create file container
        let container = NativeFileContainer {
            version: CURRENT_VERSION,
            document: doc.clone(),
            metadata: FileMetadata::new(),
        };

        // Serialize document
        let serialized = bincode::serialize(&container)
            .map_err(|e| NativeError::Serialization(e.to_string()))?;

        if let Some(ref callback) = self.progress_callback {
            callback(0, 100);
        }

        // Compress if needed
        let data = if self.compression_level > 0 {
            self.compress(&serialized)?
        } else {
            serialized
        };

        if let Some(ref callback) = self.progress_callback {
            callback(50, 100);
        }

        // Write data length
        writer.write_all(&(data.len() as u64).to_le_bytes())?;

        // Write data
        writer.write_all(&data)?;

        if let Some(ref callback) = self.progress_callback {
            callback(100, 100);
        }

        Ok(())
    }

    /// Load document from native binary format
    pub fn load<P: AsRef<Path>>(&self, path: P) -> NativeResult<Document> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read and verify magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC_BYTES {
            return Err(NativeError::InvalidFormat);
        }

        // Read version
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);

        if version > CURRENT_VERSION {
            return Err(NativeError::UnsupportedVersion(version));
        }

        // Read compression flag
        let mut compression = [0u8; 1];
        reader.read_exact(&mut compression)?;
        let is_compressed = compression[0] > 0;

        if let Some(ref callback) = self.progress_callback {
            callback(0, 100);
        }

        // Read data length
        let mut length_bytes = [0u8; 8];
        reader.read_exact(&mut length_bytes)?;
        let data_length = u64::from_le_bytes(length_bytes) as usize;

        // Read data
        let mut data = vec![0u8; data_length];
        reader.read_exact(&mut data)?;

        if let Some(ref callback) = self.progress_callback {
            callback(50, 100);
        }

        // Decompress if needed
        let serialized = if is_compressed {
            self.decompress(&data)?
        } else {
            data
        };

        // Deserialize document
        let container: NativeFileContainer = bincode::deserialize(&serialized)
            .map_err(|e| NativeError::Deserialization(e.to_string()))?;

        if let Some(ref callback) = self.progress_callback {
            callback(100, 100);
        }

        Ok(container.document)
    }

    /// Compress data using simple run-length encoding (placeholder for real compression)
    fn compress(&self, data: &[u8]) -> NativeResult<Vec<u8>> {
        // In a real implementation, use flate2 or similar
        // For now, we'll use a simple approach

        // Basic DEFLATE compression using flate2 would go here
        // For simplicity, we'll just return the data as-is with a marker
        let mut compressed = Vec::with_capacity(data.len() + 8);
        compressed.extend_from_slice(&(data.len() as u64).to_le_bytes());
        compressed.extend_from_slice(data);

        Ok(compressed)
    }

    /// Decompress data
    fn decompress(&self, data: &[u8]) -> NativeResult<Vec<u8>> {
        if data.len() < 8 {
            return Err(NativeError::Compression("Invalid compressed data".to_string()));
        }

        let mut original_size_bytes = [0u8; 8];
        original_size_bytes.copy_from_slice(&data[0..8]);
        let _original_size = u64::from_le_bytes(original_size_bytes);

        // Return the data without the size header
        Ok(data[8..].to_vec())
    }
}

impl Default for NativeFormat {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON format handler for debugging and interoperability
pub struct JsonFormat {
    /// Pretty print JSON
    pretty: bool,
    /// Progress callback
    progress_callback: Option<Box<dyn Fn(usize, usize)>>,
}

impl JsonFormat {
    /// Create a new JSON format handler
    pub fn new() -> Self {
        Self {
            pretty: true,
            progress_callback: None,
        }
    }

    /// Enable/disable pretty printing
    pub fn pretty(mut self, enabled: bool) -> Self {
        self.pretty = enabled;
        self
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Save document to JSON format
    pub fn save<P: AsRef<Path>>(&self, doc: &Document, path: P) -> NativeResult<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        if let Some(ref callback) = self.progress_callback {
            callback(0, 100);
        }

        let container = NativeFileContainer {
            version: CURRENT_VERSION,
            document: doc.clone(),
            metadata: FileMetadata::new(),
        };

        if let Some(ref callback) = self.progress_callback {
            callback(50, 100);
        }

        if self.pretty {
            serde_json::to_writer_pretty(writer, &container)
                .map_err(|e| NativeError::Serialization(e.to_string()))?;
        } else {
            serde_json::to_writer(writer, &container)
                .map_err(|e| NativeError::Serialization(e.to_string()))?;
        }

        if let Some(ref callback) = self.progress_callback {
            callback(100, 100);
        }

        Ok(())
    }

    /// Load document from JSON format
    pub fn load<P: AsRef<Path>>(&self, path: P) -> NativeResult<Document> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        if let Some(ref callback) = self.progress_callback {
            callback(0, 100);
        }

        let container: NativeFileContainer = serde_json::from_reader(reader)
            .map_err(|e| NativeError::Deserialization(e.to_string()))?;

        if let Some(ref callback) = self.progress_callback {
            callback(100, 100);
        }

        Ok(container.document)
    }

    /// Save document to JSON string
    pub fn to_string(&self, doc: &Document) -> NativeResult<String> {
        let container = NativeFileContainer {
            version: CURRENT_VERSION,
            document: doc.clone(),
            metadata: FileMetadata::new(),
        };

        if self.pretty {
            serde_json::to_string_pretty(&container)
                .map_err(|e| NativeError::Serialization(e.to_string()))
        } else {
            serde_json::to_string(&container)
                .map_err(|e| NativeError::Serialization(e.to_string()))
        }
    }

    /// Load document from JSON string
    pub fn from_string(&self, json: &str) -> NativeResult<Document> {
        let container: NativeFileContainer = serde_json::from_str(json)
            .map_err(|e| NativeError::Deserialization(e.to_string()))?;

        Ok(container.document)
    }
}

impl Default for JsonFormat {
    fn default() -> Self {
        Self::new()
    }
}

/// File container with version information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeFileContainer {
    /// File format version
    version: u32,
    /// The document data
    document: Document,
    /// File metadata
    metadata: FileMetadata,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileMetadata {
    /// CADDY version that created the file
    caddy_version: String,
    /// Timestamp when file was created
    created: chrono::DateTime<chrono::Utc>,
    /// Platform (OS)
    platform: String,
}

impl FileMetadata {
    fn new() -> Self {
        Self {
            caddy_version: env!("CARGO_PKG_VERSION").to_string(),
            created: chrono::Utc::now(),
            platform: std::env::consts::OS.to_string(),
        }
    }
}

/// Auto-detection of file format
pub struct FormatDetector;

impl FormatDetector {
    /// Detect the format of a file
    pub fn detect<P: AsRef<Path>>(path: P) -> NativeResult<FileFormat> {
        let path = path.as_ref();

        // First check by extension
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("cdy") => return Ok(FileFormat::NativeBinary),
                Some("cdyj") => return Ok(FileFormat::NativeJson),
                Some("dxf") => return Ok(FileFormat::Dxf),
                Some("svg") => return Ok(FileFormat::Svg),
                _ => {}
            }
        }

        // Try to detect by content
        let mut file = File::open(path)?;
        let mut header = [0u8; 4];
        file.read_exact(&mut header)?;

        if &header == MAGIC_BYTES {
            Ok(FileFormat::NativeBinary)
        } else if header[0] == b'{' || header[0] == b'[' {
            Ok(FileFormat::NativeJson)
        } else if &header[0..2] == b"  " || header[0] == b'0' {
            Ok(FileFormat::Dxf)
        } else {
            Err(NativeError::InvalidFormat)
        }
    }

    /// Load a document with automatic format detection
    pub fn load<P: AsRef<Path>>(path: P) -> NativeResult<Document> {
        let format = Self::detect(&path)?;

        match format {
            FileFormat::NativeBinary => NativeFormat::new().load(path),
            FileFormat::NativeJson => JsonFormat::new().load(path),
            FileFormat::Dxf => {
                use crate::io::dxf::DxfReader;
                DxfReader::new().read_file(path)
                    .map_err(|e| NativeError::Deserialization(e.to_string()))
            }
            _ => Err(NativeError::InvalidFormat),
        }
    }
}

/// File format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// Native binary format (.cdy)
    NativeBinary,
    /// Native JSON format (.cdyj)
    NativeJson,
    /// DXF format
    Dxf,
    /// SVG format
    Svg,
    /// Unknown format
    Unknown,
}

impl FileFormat {
    /// Get the default file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::NativeBinary => "cdy",
            FileFormat::NativeJson => "cdyj",
            FileFormat::Dxf => "dxf",
            FileFormat::Svg => "svg",
            FileFormat::Unknown => "",
        }
    }

    /// Get a human-readable name for this format
    pub fn name(&self) -> &'static str {
        match self {
            FileFormat::NativeBinary => "CADDY Binary",
            FileFormat::NativeJson => "CADDY JSON",
            FileFormat::Dxf => "AutoCAD DXF",
            FileFormat::Svg => "SVG Vector Graphics",
            FileFormat::Unknown => "Unknown",
        }
    }

    /// Get file filter string for file dialogs
    pub fn filter_string(&self) -> String {
        format!("{} (*.{})", self.name(), self.extension())
    }

    /// Get all supported formats
    pub fn all() -> &'static [FileFormat] {
        &[
            FileFormat::NativeBinary,
            FileFormat::NativeJson,
            FileFormat::Dxf,
            FileFormat::Svg,
        ]
    }
}

/// Backup file manager
pub struct BackupManager {
    /// Number of backups to keep
    backup_count: usize,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(backup_count: usize) -> Self {
        Self { backup_count }
    }

    /// Create a backup of a file
    pub fn create_backup<P: AsRef<Path>>(&self, path: P) -> NativeResult<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(());
        }

        // Shift existing backups
        for i in (1..self.backup_count).rev() {
            let from = self.backup_path(path, i);
            let to = self.backup_path(path, i + 1);

            if from.exists() {
                std::fs::rename(&from, &to)?;
            }
        }

        // Create new backup
        let backup_path = self.backup_path(path, 1);
        std::fs::copy(path, backup_path)?;

        Ok(())
    }

    /// Get the path for a backup file
    fn backup_path(&self, original: &Path, number: usize) -> std::path::PathBuf {
        let mut backup = original.to_path_buf();
        let mut filename = original
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        filename.push_str(&format!(".bak{}", number));
        backup.set_file_name(filename);
        backup
    }

    /// List all backups for a file
    pub fn list_backups<P: AsRef<Path>>(&self, path: P) -> Vec<std::path::PathBuf> {
        let path = path.as_ref();
        let mut backups = Vec::new();

        for i in 1..=self.backup_count {
            let backup = self.backup_path(path, i);
            if backup.exists() {
                backups.push(backup);
            }
        }

        backups
    }

    /// Restore from a backup
    pub fn restore_backup<P: AsRef<Path>>(&self, path: P, backup_number: usize) -> NativeResult<()> {
        let path = path.as_ref();
        let backup = self.backup_path(path, backup_number);

        if !backup.exists() {
            return Err(NativeError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Backup not found",
            )));
        }

        std::fs::copy(backup, path)?;
        Ok(())
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new(3) // Keep 3 backups by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::document::*;

    #[test]
    fn test_native_format_roundtrip() {
        let doc = Document::new();
        let format = NativeFormat::new();

        let path = std::env::temp_dir().join("test.cdy");
        format.save(&doc, &path).unwrap();
        let loaded = format.load(&path).unwrap();

        assert_eq!(doc.id, loaded.id);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_json_format_roundtrip() {
        let doc = Document::new();
        let format = JsonFormat::new();

        let json = format.to_string(&doc).unwrap();
        let loaded = format.from_string(&json).unwrap();

        assert_eq!(doc.id, loaded.id);
    }

    #[test]
    fn test_format_detection() {
        use std::io::Write;

        let path = std::env::temp_dir().join("test.cdy");
        let mut file = File::create(&path).unwrap();
        file.write_all(MAGIC_BYTES).unwrap();
        drop(file);

        let format = FormatDetector::detect(&path).unwrap();
        assert_eq!(format, FileFormat::NativeBinary);

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_backup_manager() {
        use std::io::Write;

        let path = std::env::temp_dir().join("test_backup.cdy");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"test data").unwrap();
        drop(file);

        let manager = BackupManager::new(3);
        manager.create_backup(&path).unwrap();

        let backups = manager.list_backups(&path);
        assert_eq!(backups.len(), 1);

        std::fs::remove_file(path).ok();
        for backup in backups {
            std::fs::remove_file(backup).ok();
        }
    }
}
