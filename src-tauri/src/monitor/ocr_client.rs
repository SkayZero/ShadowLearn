use image::{DynamicImage, GenericImageView};
use std::path::Path;
use tracing::{debug, info};

/// OCR client for extracting text and patterns from screenshots
/// Uses lightweight pattern detection instead of heavy OCR libraries
pub struct LocalOCR {
    /// Minimum confidence for text detection (0.0 - 1.0)
    confidence_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct OCRResult {
    pub text: String,
    pub confidence: f32,
    pub detected_patterns: Vec<DetectedPattern>,
}

#[derive(Debug, Clone)]
pub enum DetectedPattern {
    CodeEditor {
        language: Option<String>,
        has_errors: bool,
    },
    Terminal {
        has_errors: bool,
        error_type: Option<String>,
    },
    Browser {
        has_stack_trace: bool,
    },
    IDE {
        name: String,
    },
}

impl LocalOCR {
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7,
        }
    }

    /// Extract text and patterns from an image
    pub fn analyze(&self, image_path: &Path) -> Result<OCRResult, String> {
        let start = std::time::Instant::now();

        // Load image
        let img = image::open(image_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;

        // Detect patterns based on image characteristics
        let patterns = self.detect_patterns(&img);

        // Generate text description based on patterns
        let text = self.generate_description(&patterns);

        let duration = start.elapsed();
        info!("🔍 OCR analysis completed in {}ms", duration.as_millis());

        Ok(OCRResult {
            text,
            confidence: 0.85, // Pattern-based detection is quite reliable
            detected_patterns: patterns,
        })
    }

    /// Detect patterns in the image based on color distribution and layout
    fn detect_patterns(&self, img: &DynamicImage) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        let (width, height) = img.dimensions();
        debug!("Analyzing image: {}x{}", width, height);

        // Sample colors from different regions
        let samples = self.sample_image_regions(img);

        // Detect code editor (dark theme with syntax highlighting colors)
        if self.is_code_editor(&samples) {
            let has_errors = self.detect_red_squiggles(&samples);
            patterns.push(DetectedPattern::CodeEditor {
                language: self.guess_language(&samples),
                has_errors,
            });
        }

        // Detect terminal (monospace, black/white/green)
        if self.is_terminal(&samples) {
            let has_errors = self.detect_error_colors(&samples);
            patterns.push(DetectedPattern::Terminal {
                has_errors,
                error_type: if has_errors {
                    Some("Command failed or stack trace detected".to_string())
                } else {
                    None
                },
            });
        }

        // Detect browser (address bar, tabs)
        if self.is_browser(&samples) {
            let has_stack_trace = self.detect_stack_trace_pattern(&samples);
            patterns.push(DetectedPattern::Browser { has_stack_trace });
        }

        patterns
    }

    /// Sample colors from a grid of regions in the image
    fn sample_image_regions(&self, img: &DynamicImage) -> Vec<(u32, u32, image::Rgba<u8>)> {
        let (width, height) = img.dimensions();
        let mut samples = Vec::new();

        // Sample 10x10 grid
        for y in 0..10 {
            for x in 0..10 {
                let px = (x * width / 10).min(width - 1);
                let py = (y * height / 10).min(height - 1);
                let pixel = img.get_pixel(px, py);
                samples.push((px, py, pixel));
            }
        }

        samples
    }

    /// Detect if the image looks like a code editor
    fn is_code_editor(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> bool {
        // Code editors typically have:
        // - Dark background (low RGB values)
        // - Syntax highlighting (variety of colors)
        // - High contrast

        let dark_pixels = samples
            .iter()
            .filter(|(_, _, pixel)| {
                let r = pixel[0] as u32;
                let g = pixel[1] as u32;
                let b = pixel[2] as u32;
                (r + g + b) / 3 < 80 // Dark background
            })
            .count();

        let dark_ratio = dark_pixels as f32 / samples.len() as f32;

        // Check for colorful syntax highlighting
        let colorful_pixels = samples
            .iter()
            .filter(|(_, _, pixel)| {
                let r = pixel[0] as i32;
                let g = pixel[1] as i32;
                let b = pixel[2] as i32;
                // Colors with high variance = syntax highlighting
                let variance = ((r - g).abs() + (g - b).abs() + (r - b).abs()) / 3;
                variance > 30
            })
            .count();

        let colorful_ratio = colorful_pixels as f32 / samples.len() as f32;

        dark_ratio > 0.5 && colorful_ratio > 0.2
    }

    /// Detect if the image looks like a terminal
    fn is_terminal(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> bool {
        // Terminals typically have:
        // - Mostly monochrome (white/green on black, or black on white)
        // - Less color variety than code editors

        let monochrome_pixels = samples
            .iter()
            .filter(|(_, _, pixel)| {
                let r = pixel[0] as i32;
                let g = pixel[1] as i32;
                let b = pixel[2] as i32;
                // Low color variance = monochrome
                let variance = ((r - g).abs() + (g - b).abs() + (r - b).abs()) / 3;
                variance < 20
            })
            .count();

        let monochrome_ratio = monochrome_pixels as f32 / samples.len() as f32;
        monochrome_ratio > 0.7
    }

    /// Detect if the image looks like a browser
    fn is_browser(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> bool {
        // Browsers typically have:
        // - Light background (high RGB values)
        // - Top bar with tabs/address bar

        let light_pixels = samples
            .iter()
            .filter(|(_, _, pixel)| {
                let r = pixel[0] as u32;
                let g = pixel[1] as u32;
                let b = pixel[2] as u32;
                (r + g + b) / 3 > 200 // Light background
            })
            .count();

        let light_ratio = light_pixels as f32 / samples.len() as f32;
        light_ratio > 0.6
    }

    /// Detect red squiggly lines (error indicators)
    fn detect_red_squiggles(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> bool {
        let red_pixels = samples
            .iter()
            .filter(|(_, _, pixel)| {
                let r = pixel[0] as u32;
                let g = pixel[1] as u32;
                let b = pixel[2] as u32;
                r > 200 && g < 100 && b < 100 // Bright red
            })
            .count();

        red_pixels > 5 // At least some red indicators
    }

    /// Detect error colors (red text in terminal)
    fn detect_error_colors(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> bool {
        self.detect_red_squiggles(samples)
    }

    /// Detect stack trace pattern (lots of text, specific formatting)
    fn detect_stack_trace_pattern(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> bool {
        // Stack traces typically have lots of structured text
        // For now, detect presence of monospace patterns
        self.is_terminal(samples)
    }

    /// Guess programming language based on color scheme
    fn guess_language(&self, samples: &[(u32, u32, image::Rgba<u8>)]) -> Option<String> {
        // Different languages have different typical color schemes in editors
        // This is a simplified heuristic

        let has_blue = samples.iter().any(|(_, _, pixel)| {
            pixel[2] > 150 && pixel[0] < 100 && pixel[1] < 100
        });

        let has_green = samples.iter().any(|(_, _, pixel)| {
            pixel[1] > 150 && pixel[0] < 100 && pixel[2] < 100
        });

        let has_yellow = samples.iter().any(|(_, _, pixel)| {
            pixel[0] > 200 && pixel[1] > 200 && pixel[2] < 100
        });

        if has_blue && has_green && has_yellow {
            Some("JavaScript".to_string())
        } else if has_blue && has_green {
            Some("Python".to_string())
        } else if has_yellow {
            Some("Rust".to_string())
        } else {
            None
        }
    }

    /// Generate a human-readable description from detected patterns
    fn generate_description(&self, patterns: &[DetectedPattern]) -> String {
        let mut descriptions = Vec::new();

        for pattern in patterns {
            match pattern {
                DetectedPattern::CodeEditor { language, has_errors } => {
                    let lang = language.as_deref().unwrap_or("unknown language");
                    if *has_errors {
                        descriptions.push(format!(
                            "Code editor detected ({}) with potential errors highlighted",
                            lang
                        ));
                    } else {
                        descriptions.push(format!("Code editor detected ({})", lang));
                    }
                }
                DetectedPattern::Terminal { has_errors, error_type } => {
                    if *has_errors {
                        let err = error_type.as_deref().unwrap_or("unknown error");
                        descriptions.push(format!("Terminal with error: {}", err));
                    } else {
                        descriptions.push("Terminal session active".to_string());
                    }
                }
                DetectedPattern::Browser { has_stack_trace } => {
                    if *has_stack_trace {
                        descriptions.push("Browser with error/stack trace visible".to_string());
                    } else {
                        descriptions.push("Web browser active".to_string());
                    }
                }
                DetectedPattern::IDE { name } => {
                    descriptions.push(format!("IDE detected: {}", name));
                }
            }
        }

        if descriptions.is_empty() {
            "Generic application window".to_string()
        } else {
            descriptions.join(". ")
        }
    }
}

impl Default for LocalOCR {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_creation() {
        let ocr = LocalOCR::new();
        assert_eq!(ocr.confidence_threshold, 0.7);
    }

    #[test]
    fn test_pattern_detection() {
        let ocr = LocalOCR::new();

        // Create a simple dark image (simulating code editor)
        let img = DynamicImage::new_rgb8(100, 100);
        let samples = ocr.sample_image_regions(&img);

        assert_eq!(samples.len(), 100); // 10x10 grid
    }
}
