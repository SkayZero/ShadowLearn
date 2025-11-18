use image::RgbImage;
use std::path::PathBuf;
use tracing::{debug, info};

/// Détecte les changements significatifs entre deux screenshots
pub struct ChangeDetector {
    last_hash: Option<u64>,
    similarity_threshold: f32, // 0.0 = différent, 1.0 = identique
}

impl ChangeDetector {
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            last_hash: None,
            similarity_threshold,
        }
    }

    /// Détecte si un changement significatif s'est produit
    pub fn has_significant_change(&mut self, image_path: &PathBuf) -> Result<bool, String> {
        let current_hash = self.calculate_perceptual_hash(image_path)?;

        let is_different = if let Some(last) = self.last_hash {
            let similarity = self.hash_similarity(last, current_hash);
            debug!("🔍 Similarity: {:.2}% (threshold: {:.2}%)",
                   similarity * 100.0,
                   self.similarity_threshold * 100.0);

            similarity < self.similarity_threshold
        } else {
            // Premier screenshot = toujours un changement
            true
        };

        if is_different {
            info!("✨ Changement significatif détecté!");
            self.last_hash = Some(current_hash);
        }

        Ok(is_different)
    }

    /// Calcule un hash perceptuel simple (average hash)
    /// Plus rapide que pHash mais suffisant pour nos besoins
    fn calculate_perceptual_hash(&self, image_path: &PathBuf) -> Result<u64, String> {
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        // Redimensionner à 8x8 pour le hash
        let small = img.resize_exact(8, 8, image::imageops::FilterType::Triangle);
        let gray = small.to_luma8();

        // Calculer la moyenne des pixels
        let sum: u32 = gray.pixels().map(|p| p.0[0] as u32).sum();
        let avg = sum / 64;

        // Générer le hash binaire
        let mut hash: u64 = 0;
        for (i, pixel) in gray.pixels().enumerate() {
            if pixel.0[0] as u32 > avg {
                hash |= 1 << i;
            }
        }

        Ok(hash)
    }

    /// Calcule la similarité entre deux hash (0.0 = différent, 1.0 = identique)
    fn hash_similarity(&self, hash1: u64, hash2: u64) -> f32 {
        let diff_bits = (hash1 ^ hash2).count_ones();
        1.0 - (diff_bits as f32 / 64.0)
    }

    /// Reset le détecteur (utile après un changement d'app ou contexte)
    pub fn reset(&mut self) {
        self.last_hash = None;
        info!("🔄 Change detector reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_similarity() {
        let detector = ChangeDetector::new(0.85);

        // Hash identiques
        assert_eq!(detector.hash_similarity(0xFF, 0xFF), 1.0);

        // Hash complètement différents
        assert_eq!(detector.hash_similarity(0x00, 0xFF), 0.0);

        // Hash avec 1 bit différent
        let sim = detector.hash_similarity(0xFF, 0xFE);
        assert!(sim > 0.98); // 63/64 bits identiques
    }
}
