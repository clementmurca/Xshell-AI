use crate::color::{Color, ColorScheme};
use crate::error::{Result, ThemeError};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

/// Parse un `.itermcolors` depuis un chemin de fichier.
pub fn parse_file(path: &Path, name: &str) -> Result<ColorScheme> {
    let file = std::fs::File::open(path).map_err(|source| ThemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_reader(file, name, path)
}

/// Parse un `.itermcolors` depuis n'importe quel `Read`. `path` sert au reporting d'erreur.
pub fn parse_reader<R: Read>(reader: R, name: &str, path: &Path) -> Result<ColorScheme> {
    let raw: HashMap<String, plist::Value> =
        plist::from_reader_xml(reader).map_err(|source| ThemeError::Plist {
            path: path.to_path_buf(),
            source,
        })?;

    let mut ansi = [Color::new(0, 0, 0); 16];
    for (i, slot) in ansi.iter_mut().enumerate() {
        let key = format!("Ansi {i} Color");
        *slot = extract_color(&raw, &key)?;
    }

    let foreground = extract_color(&raw, "Foreground Color")?;
    let background = extract_color(&raw, "Background Color")?;
    let cursor = extract_optional_color(&raw, "Cursor Color")?;
    let selection_background = extract_optional_color(&raw, "Selection Color")?;

    Ok(ColorScheme {
        name: name.to_string(),
        ansi,
        foreground,
        background,
        cursor,
        selection_background,
    })
}

fn extract_color(map: &HashMap<String, plist::Value>, key: &str) -> Result<Color> {
    extract_optional_color(map, key)?.ok_or_else(|| ThemeError::MissingKey(key.to_string()))
}

fn extract_optional_color(
    map: &HashMap<String, plist::Value>,
    key: &str,
) -> Result<Option<Color>> {
    let Some(value) = map.get(key) else {
        return Ok(None);
    };
    let dict = value.as_dictionary().ok_or(ThemeError::InvalidColor {
        key: key.to_string(),
        reason: "not a dictionary".to_string(),
    })?;
    let r = read_component(dict, "Red Component", key)?;
    let g = read_component(dict, "Green Component", key)?;
    let b = read_component(dict, "Blue Component", key)?;
    Ok(Some(Color::from_f32(r as f32, g as f32, b as f32)))
}

fn read_component(dict: &plist::Dictionary, component: &str, key: &str) -> Result<f64> {
    dict.get(component)
        .and_then(|v| v.as_real())
        .ok_or_else(|| ThemeError::InvalidColor {
            key: key.to_string(),
            reason: format!("missing or invalid {component}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dracula_fixture() {
        let path = Path::new("tests/fixtures/dracula.itermcolors");
        let scheme = parse_file(path, "dracula").expect("parse");
        assert_eq!(scheme.name, "dracula");
        assert_eq!(scheme.background, Color::new(40, 42, 54));
        assert_eq!(scheme.foreground, Color::new(248, 248, 242));
        assert!(scheme.cursor.is_some());
    }

    #[test]
    fn missing_key_errors_out() {
        let xml = r#"<?xml version="1.0"?>
<plist version="1.0"><dict>
  <key>Foreground Color</key>
  <dict><key>Red Component</key><real>1.0</real>
  <key>Green Component</key><real>1.0</real>
  <key>Blue Component</key><real>1.0</real></dict>
</dict></plist>"#;
        let err = parse_reader(xml.as_bytes(), "bad", Path::new("inline")).unwrap_err();
        assert!(matches!(err, ThemeError::MissingKey(k) if k == "Ansi 0 Color"));
    }
}
