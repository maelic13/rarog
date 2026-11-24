//! Helpers the integration tests share, taken with `mod common;`.

/// Owned words for a parser that takes `&[String]`.
pub fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}
