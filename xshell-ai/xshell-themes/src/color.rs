use serde::{Deserialize, Serialize};

/// Couleur RGB sur 8 bits par canal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Construit à partir de trois composantes normalisées [0.0, 1.0] (format iTerm2).
    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            g: (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            b: (b.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }

    /// Formatte en `#rrggbb`.
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Scheme complet : 16 ANSI + foreground + background + curseur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorScheme {
    pub name: String,
    pub ansi: [Color; 16],
    pub foreground: Color,
    pub background: Color,
    pub cursor: Option<Color>,
    pub selection_background: Option<Color>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_f32_converts_with_rounding() {
        let c = Color::from_f32(1.0, 0.5, 0.0);
        assert_eq!(c, Color::new(255, 128, 0));
    }

    #[test]
    fn from_f32_clamps_out_of_range() {
        let c = Color::from_f32(-0.5, 2.0, 0.25);
        assert_eq!(c, Color::new(0, 255, 64));
    }

    #[test]
    fn to_hex_formats_correctly() {
        assert_eq!(Color::new(255, 128, 0).to_hex(), "#ff8000");
        assert_eq!(Color::new(0, 0, 0).to_hex(), "#000000");
    }
}
