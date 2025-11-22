/**
 * Pattern Detection for Phase 3B
 *
 * Implements intelligent pattern detection algorithms:
 * - Refacto: Detects code duplication using sliding window
 * - Debug: Detects debugging patterns (console.log, println!, etc.)
 */

use regex::Regex;
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

/// Minimum lines for code duplication detection
const MIN_DUPLICATE_LINES: usize = 3;

/// Minimum similarity threshold for duplication (0.0 - 1.0)
const SIMILARITY_THRESHOLD: f64 = 0.85;

/// Detect refacto opportunity (code duplication)
pub fn detect_refacto_pattern(
    content: &str,
    file_name: &str,
) -> Option<serde_json::Value> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.len() < MIN_DUPLICATE_LINES * 2 {
        return None; // File too small for meaningful duplication
    }

    // Use sliding window to find duplicate code blocks
    for window_size in (MIN_DUPLICATE_LINES..=10).rev() {
        if window_size > lines.len() / 2 {
            continue;
        }

        let mut blocks: HashMap<String, Vec<usize>> = HashMap::new();

        // Create sliding windows
        for (i, window) in lines.windows(window_size).enumerate() {
            let normalized = normalize_code_block(window);
            blocks.entry(normalized).or_insert_with(Vec::new).push(i);
        }

        // Check for duplicates
        for (block, positions) in blocks {
            if positions.len() >= 2 && !block.trim().is_empty() {
                // Found duplication!
                let line1 = positions[0] + 1;
                let line2 = positions[1] + 1;

                let snippet = lines[positions[0]..positions[0] + window_size]
                    .join("\n")
                    .chars()
                    .take(200)
                    .collect::<String>();

                info!(
                    "🔧 Code duplication detected: {} lines repeated at lines {} and {}",
                    window_size, line1, line2
                );

                return Some(json!({
                    "id": format!("refacto_{}", uuid::Uuid::new_v4()),
                    "type": "refacto",
                    "title": "Code répété détecté",
                    "description": format!(
                        "J'ai détecté {} lignes de code répétées dans ce fichier. \
                        Vous pourriez extraire cette logique dans une fonction pour améliorer \
                        la maintenabilité.",
                        window_size
                    ),
                    "confidence": calculate_confidence(window_size, positions.len()),
                    "status": "pending",
                    "timestamp": chrono::Utc::now().timestamp(),
                    "context": {
                        "app": "Code Editor",
                        "file": file_name,
                        "line": line1,
                        "codeSnippet": snippet,
                        "duplicateLocations": positions.iter().map(|p| p + 1).collect::<Vec<_>>(),
                    }
                }));
            }
        }
    }

    None
}

/// Detect debug opportunity (excessive console.log, println!, etc.)
pub fn detect_debug_pattern(
    content: &str,
    file_name: &str,
) -> Option<serde_json::Value> {
    // Regex patterns for debug statements
    let patterns = [
        (r"console\.(log|debug|info|warn|error)", "JavaScript/TypeScript"),
        (r"println!\s*\(", "Rust"),
        (r"print\s*\(", "Python"),
        (r"System\.out\.println", "Java"),
        (r"fmt\.Println", "Go"),
        (r"std::cout\s*<<", "C++"),
    ];

    for (pattern_str, lang) in patterns {
        if let Ok(regex) = Regex::new(pattern_str) {
            let matches: Vec<_> = regex.find_iter(content).collect();

            // If 3+ debug statements, suggest using a debugger
            if matches.len() >= 3 {
                // Find first occurrence for context
                let first_match = matches[0];
                let line_num = content[..first_match.start()]
                    .lines()
                    .count()
                    + 1;

                // Extract snippet around first match
                let lines: Vec<&str> = content.lines().collect();
                let start_line = line_num.saturating_sub(2);
                let end_line = (line_num + 1).min(lines.len());
                let snippet = lines[start_line..end_line]
                    .join("\n")
                    .chars()
                    .take(200)
                    .collect::<String>();

                info!(
                    "🐛 Debug pattern detected: {} debug statements in {} code",
                    matches.len(),
                    lang
                );

                return Some(json!({
                    "id": format!("debug_{}", uuid::Uuid::new_v4()),
                    "type": "debug",
                    "title": "Nombreux statements de debug détectés",
                    "description": format!(
                        "J'ai trouvé {} statements de debug ({}) dans ce fichier. \
                        Considérez utiliser un vrai debugger ou un système de logging structuré \
                        pour une meilleure efficacité.",
                        matches.len(),
                        lang
                    ),
                    "confidence": calculate_debug_confidence(matches.len()),
                    "status": "pending",
                    "timestamp": chrono::Utc::now().timestamp(),
                    "context": {
                        "app": "Code Editor",
                        "file": file_name,
                        "line": line_num,
                        "codeSnippet": snippet,
                        "debugCount": matches.len(),
                        "language": lang,
                    }
                }));
            }
        }
    }

    None
}

/// Normalize a code block for comparison (remove whitespace, comments)
fn normalize_code_block(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| {
            // Remove leading/trailing whitespace
            let trimmed = line.trim();
            // Remove single-line comments (basic)
            if let Some(pos) = trimmed.find("//") {
                &trimmed[..pos]
            } else {
                trimmed
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Calculate confidence based on duplication size and count
fn calculate_confidence(window_size: usize, duplicate_count: usize) -> f64 {
    let base_confidence = 0.7;
    let size_bonus = (window_size as f64 / 10.0).min(0.2);
    let count_bonus = ((duplicate_count - 2) as f64 * 0.05).min(0.1);

    (base_confidence + size_bonus + count_bonus).min(1.0)
}

/// Calculate confidence for debug detection based on count
fn calculate_debug_confidence(debug_count: usize) -> f64 {
    let base_confidence = 0.75;
    let count_bonus = ((debug_count - 3) as f64 * 0.05).min(0.2);

    (base_confidence + count_bonus).min(0.95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_refacto_simple() {
        let code = r#"
fn process_a() {
    let x = 5;
    let y = 10;
    println!("Result: {}", x + y);
}

fn process_b() {
    let x = 5;
    let y = 10;
    println!("Result: {}", x + y);
}
"#;

        let result = detect_refacto_pattern(code, "test.rs");
        assert!(result.is_some());
    }

    #[test]
    fn test_detect_debug_rust() {
        let code = r#"
fn main() {
    println!("Debug 1");
    let x = 5;
    println!("Debug 2");
    let y = 10;
    println!("Debug 3");
}
"#;

        let result = detect_debug_pattern(code, "test.rs");
        assert!(result.is_some());
    }

    #[test]
    fn test_detect_debug_javascript() {
        let code = r#"
function test() {
    console.log("Debug 1");
    const x = 5;
    console.log("Debug 2");
    const y = 10;
    console.log("Debug 3");
    console.error("Debug 4");
}
"#;

        let result = detect_debug_pattern(code, "test.js");
        assert!(result.is_some());
    }
}
