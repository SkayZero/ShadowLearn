use base64::{engine::general_purpose, Engine as _};
use flate2::{read::GzEncoder, Compression};
use std::io::Read;
use tracing::debug;

/// Gestionnaire de compression pour optimiser la taille des données
pub struct CompressionManager {
    compression_level: Compression,
}

impl CompressionManager {
    pub fn new() -> Self {
        Self {
            compression_level: Compression::default(),
        }
    }

    /// Compresse une chaîne de caractères
    pub fn compress_string(&self, data: &str) -> Result<Vec<u8>, String> {
        let mut encoder = GzEncoder::new(data.as_bytes(), self.compression_level);
        let mut compressed = Vec::new();

        encoder
            .read_to_end(&mut compressed)
            .map_err(|e| format!("Compression failed: {}", e))?;

        debug!(
            "📦 Compressed {} bytes to {} bytes ({:.1}% reduction)",
            data.len(),
            compressed.len(),
            (1.0 - compressed.len() as f64 / data.len() as f64) * 100.0
        );

        Ok(compressed)
    }

    /// Compresse une chaîne et la convertit en base64
    pub fn compress_to_base64(&self, data: &str) -> Result<String, String> {
        let compressed = self.compress_string(data)?;
        Ok(general_purpose::STANDARD.encode(compressed))
    }

    /// Détermine si la compression est bénéfique pour cette taille de données
    pub fn should_compress(&self, data_size: usize) -> bool {
        // Compresser seulement si les données font plus de 1KB
        data_size > 1024
    }
}

impl Default for CompressionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_decompression() {
        let manager = CompressionManager::new();
        let test_data = "This is a test string that should compress well because it has repeated patterns and is long enough to benefit from compression.";

        // Test compression
        let compressed = manager.compress_string(test_data).unwrap();
        assert!(compressed.len() < test_data.len());

        // Test decompression
        let decompressed = manager.decompress_string(&compressed).unwrap();
        assert_eq!(decompressed, test_data);
    }

    #[test]
    fn test_base64_compression() {
        let manager = CompressionManager::new();
        let test_data = "This is a test string for base64 compression.";

        let compressed_b64 = manager.compress_to_base64(test_data).unwrap();
        let decompressed = manager.decompress_from_base64(&compressed_b64).unwrap();

        assert_eq!(decompressed, test_data);
    }

    #[test]
    fn test_should_compress() {
        let manager = CompressionManager::new();

        assert!(!manager.should_compress(500)); // Small data
        assert!(manager.should_compress(2000)); // Large data
    }
}
