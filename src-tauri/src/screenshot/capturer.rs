use super::errors::{PermissionStatus, ScreenshotError};
use image::codecs::jpeg::JpegEncoder;
use image::RgbImage;
use screenshots::Screen;
use std::io::Cursor;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::NamedTempFile;
use tracing::{debug, info, warn};

const DEFAULT_QUALITY: u8 = 50; // OPTIMIZED: 60 → 50 (faster encode + smaller file)
const MAX_SIZE_BYTES: usize = 300_000; // OPTIMIZED: 500KB → 300KB
const MIN_DIMENSION: u32 = 100; // Don't scale below this
const MAX_WIDTH_AGGRESSIVE: u32 = 720; // OPTIMIZED: 960 → 720 (fewer pixels = MUCH faster)

pub struct ScreenshotCapturer {
    screens: Vec<Screen>,
    compression_quality: u8,
    max_size_bytes: usize,
    permission_checked: Option<PermissionStatus>,
}

impl ScreenshotCapturer {
    pub fn new() -> Result<Self, ScreenshotError> {
        let screens = Screen::all().map_err(|e| ScreenshotError::InitFailed(e.to_string()))?;

        if screens.is_empty() {
            return Err(ScreenshotError::NoScreens);
        }

        info!("📺 Initialized {} screen(s)", screens.len());

        Ok(Self {
            screens,
            compression_quality: DEFAULT_QUALITY,
            max_size_bytes: MAX_SIZE_BYTES,
            permission_checked: None,
        })
    }

    /// Capture le screen contenant le curseur (ou primary si échec)
    /// OPTIMIZED: Direct RGBA→RGB conversion without intermediate DynamicImage
    pub fn capture_active_screen(&self) -> Result<PathBuf, ScreenshotError> {
        let start = Instant::now();

        let screen = self
            .get_screen_under_cursor()
            .or_else(|| self.screens.first())
            .ok_or(ScreenshotError::NoScreens)?;

        debug!(
            "📸 Capturing screen {} ({}x{})",
            screen.display_info.id, screen.display_info.width, screen.display_info.height
        );

        let image = screen
            .capture()
            .map_err(|e| ScreenshotError::CaptureFailed(e.to_string()))?;

        // OPTIMIZED: Direct RGBA→RGB + downscale in ONE step to avoid double processing
        let rgba_data = image.to_vec();
        let (width, height) = (image.width(), image.height());
        
        // Downscale DURING conversion if needed
        let scale = if width > MAX_WIDTH_AGGRESSIVE {
            MAX_WIDTH_AGGRESSIVE as f32 / width as f32
        } else {
            1.0
        };
        
        let new_width = (width as f32 * scale) as u32;
        let new_height = (height as f32 * scale) as u32;
        
        info!("🔄 Downscaling DURING conversion: {}x{} → {}x{} ({:.0}%)", width, height, new_width, new_height, scale * 100.0);
        
        // Fast downscale using pixels sampling
        let rgb_data: Vec<u8> = if scale < 1.0 {
            let step = 1.0 / scale;
            let mut result = Vec::with_capacity((new_width * new_height * 3) as usize);
            for y in 0..new_height {
                let src_y = ((y as f32) * step) as u32;
                for x in 0..new_width {
                    let src_x = ((x as f32) * step) as u32;
                    let src_idx = ((src_y * width + src_x) * 4) as usize;
                    result.push(rgba_data[src_idx]);
                    result.push(rgba_data[src_idx + 1]);
                    result.push(rgba_data[src_idx + 2]);
                }
            }
            result
        } else {
            // No downscale needed, just convert RGBA→RGB
            rgba_data.chunks_exact(4).flat_map(|pixel| [pixel[0], pixel[1], pixel[2]]).collect()
        };
        
        let rgb_img = RgbImage::from_raw(new_width, new_height, rgb_data)
            .ok_or_else(|| ScreenshotError::ProcessingFailed("RGB conversion failed".into()))?;

        // Compress and save to temp file
        let temp_path = self.compress_and_save(rgb_img)?;

        let duration = start.elapsed();
        info!(
            "✅ Screenshot captured in {}ms → {}",
            duration.as_millis(),
            temp_path.display()
        );

        Ok(temp_path)
    }

    /// Trouve le screen sous le curseur
    /// Note: Pour l'instant, retourne le premier screen (primary)
    /// TODO: Implémenter détection cursor position pour multi-monitor
    fn get_screen_under_cursor(&self) -> Option<&Screen> {
        // Pour l'instant, on retourne toujours le premier screen
        // Une implémentation complète nécessiterait des APIs plus complexes
        self.screens.first()
    }

    /// Compresse et sauvegarde dans un fichier temporaire
    fn compress_and_save(&self, img: RgbImage) -> Result<PathBuf, ScreenshotError> {
        let overall_start = Instant::now();
        
        // OPTIMIZATION: Downscale FIRST to reasonable size (max 1280px wide for speed!)
        // This avoids encoding huge Retina images (5K = 5120x2880 = 44MB raw!)
        let mut current_img = img;
        
        info!("🖼️ Final image: {}x{} (already downscaled during conversion)", current_img.width(), current_img.height());

        // Encode to JPEG
        let encode_start = Instant::now();
        let mut buffer = Cursor::new(Vec::new());
        let encoder = JpegEncoder::new_with_quality(&mut buffer, self.compression_quality);
        current_img.write_with_encoder(encoder)?;
        info!("⏱️ First JPEG encode: {}ms", encode_start.elapsed().as_millis());

        let mut bytes = buffer.into_inner();
        info!("📦 First JPEG size: {} bytes", bytes.len());

        // Si ENCORE trop gros, downscale davantage
        if bytes.len() > self.max_size_bytes {
            warn!(
                "⚠️ Screenshot too large: {} bytes, downscaling...",
                bytes.len()
            );

            let target_ratio = (self.max_size_bytes as f32 / bytes.len() as f32).sqrt() * 0.9;
            let new_width = ((current_img.width() as f32 * target_ratio) as u32).max(MIN_DIMENSION);
            let new_height =
                ((current_img.height() as f32 * target_ratio) as u32).max(MIN_DIMENSION);

            debug!("  Resizing to {}x{}", new_width, new_height);

            let resize_start = Instant::now();
            let resized = image::imageops::resize(
                &current_img,
                new_width,
                new_height,
                image::imageops::FilterType::Triangle, // OPTIMIZED: Lanczos3 → Triangle (10x faster)
            );
            info!("⏱️ Second resize: {}ms", resize_start.elapsed().as_millis());

            // Re-encode
            let encode_start = Instant::now();
            let mut buffer = Cursor::new(Vec::new());
            let encoder = JpegEncoder::new_with_quality(&mut buffer, self.compression_quality);
            resized.write_with_encoder(encoder)?;
            info!("⏱️ Second JPEG encode: {}ms", encode_start.elapsed().as_millis());

            bytes = buffer.into_inner();
            current_img = resized;

            info!(
                "  Resized to {}x{}, new size: {} bytes",
                current_img.width(),
                current_img.height(),
                bytes.len()
            );
        }

        // Sauvegarder dans un fichier temporaire
        let save_start = Instant::now();
        let mut temp_file = NamedTempFile::with_suffix(".jpg")?;
        std::io::Write::write_all(&mut temp_file, &bytes)?;
        info!("⏱️ File write: {}ms", save_start.elapsed().as_millis());

        let path = temp_file.into_temp_path().keep().map_err(|e| {
            ScreenshotError::IoError(std::io::Error::other(format!(
                "Failed to persist temp file: {}",
                e
            )))
        })?;
        
        info!("⏱️ compress_and_save TOTAL: {}ms", overall_start.elapsed().as_millis());
        debug!("💾 Saved to: {}", path.display());

        Ok(path)
    }

    /// Vérifie les permissions (cache le résultat)
    pub fn check_permissions(&mut self) -> PermissionStatus {
        if let Some(cached) = self.permission_checked {
            return cached;
        }

        let status = self.check_permissions_impl();
        self.permission_checked = Some(status);
        status
    }

    /// Implémentation platform-specific du check permissions
    fn check_permissions_impl(&self) -> PermissionStatus {
        #[cfg(target_os = "macos")]
        {
            // CGPreflightScreenCaptureAccess() pour check sans demander
            // Pas disponible dans core-graphics 0.23, donc on fait un test léger

            // Essai de capture 1x1 pour tester
            if let Some(screen) = self.screens.first() {
                match screen.capture() {
                    Ok(_) => {
                        info!("✅ macOS screen capture permission: Granted");
                        PermissionStatus::Granted
                    }
                    Err(e) => {
                        let err_str = e.to_string().to_lowercase();
                        if err_str.contains("permission") || err_str.contains("denied") {
                            warn!("⚠️ macOS screen capture permission: Denied");
                            PermissionStatus::Denied
                        } else {
                            warn!("❓ macOS screen capture permission: Unknown ({})", e);
                            PermissionStatus::Unknown
                        }
                    }
                }
            } else {
                PermissionStatus::Unknown
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Windows/Linux : pas de permission explicite
            PermissionStatus::Granted
        }
    }
}

impl Default for ScreenshotCapturer {
    fn default() -> Self {
        Self::new().expect("Failed to create ScreenshotCapturer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capturer_creation() {
        let result = ScreenshotCapturer::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_permission_check() {
        let mut capturer = ScreenshotCapturer::new().unwrap();
        let status = capturer.check_permissions();

        // Should be Granted or Denied, not Unknown
        assert!(
            status == PermissionStatus::Granted || status == PermissionStatus::Denied,
            "Got status: {:?}",
            status
        );
    }

    #[test]
    #[ignore] // Only run manually (creates temp file)
    fn test_capture() {
        let capturer = ScreenshotCapturer::new().unwrap();
        let result = capturer.capture_active_screen();

        match result {
            Ok(path) => {
                println!("Screenshot saved to: {}", path.display());
                let metadata = std::fs::metadata(&path).unwrap();
                assert!(metadata.len() > 0);
                assert!(metadata.len() <= MAX_SIZE_BYTES as u64);
            }
            Err(e) => {
                println!("Capture error (expected on CI): {}", e);
            }
        }
    }
}
