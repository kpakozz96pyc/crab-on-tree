//! Color types and utilities.

use serde::{Deserialize, Serialize};

/// RGBA color representation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA components (0.0-1.0 range).
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a color from a hex string (#RRGGBB or #RRGGBBAA).
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#')?;

        let (r, g, b, a) = match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                (r, g, b, a)
            }
            _ => return None,
        };

        Some(Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_6_digits() {
        let color = Color::from_hex("#ffffff").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 1.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_from_hex_8_digits() {
        let color = Color::from_hex("#00000080").unwrap();
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert!((color.a - 0.502).abs() < 0.01); // 128/255 ≈ 0.502
    }

    #[test]
    fn test_from_hex_invalid() {
        assert!(Color::from_hex("#fff").is_none());
        assert!(Color::from_hex("ffffff").is_none());
        assert!(Color::from_hex("#gggggg").is_none());
    }
}
