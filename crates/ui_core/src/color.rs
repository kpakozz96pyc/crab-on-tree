//! Color types and utilities.

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// RGBA color representation, serialized as a hex string (`"#rrggbb"` / `"#rrggbbaa"`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        let a = (self.a * 255.0).round() as u8;
        if a == 255 {
            s.serialize_str(&format!("#{:02x}{:02x}{:02x}", r, g, b))
        } else {
            s.serialize_str(&format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a))
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Color::from_hex(&s).ok_or_else(|| de::Error::custom(format!("invalid hex color: {}", s)))
    }
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
