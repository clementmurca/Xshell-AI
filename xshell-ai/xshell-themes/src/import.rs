use crate::error::{Result, ThemeError};
use crate::storage;
use std::io::Read;
use std::time::Duration;

pub const MAX_BYTES: usize = 1_024 * 1_024;
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// Télécharge un `.itermcolors` depuis une URL et l'enregistre sous `name`.
pub fn import_from_url(url: &str, name: &str) -> Result<std::path::PathBuf> {
    let client = reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .build()?;
    let response = client.get(url).send()?;

    let ct = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    if !ct.contains("xml") && !ct.contains("octet-stream") && !ct.is_empty() {
        return Err(ThemeError::UnexpectedContentType(ct));
    }

    let bytes = read_bounded(response, MAX_BYTES)?;
    storage::save_raw(name, &bytes)
}

/// Lit un `Read` jusqu'à `limit + 1` octets. Retourne `SizeLimitExceeded` au-delà.
pub fn read_bounded<R: Read>(mut reader: R, limit: usize) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(limit.min(64 * 1024));
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf).map_err(|source| ThemeError::Io {
            path: std::path::PathBuf::from("<http-stream>"),
            source,
        })?;
        if n == 0 {
            break;
        }
        if out.len() + n > limit {
            return Err(ThemeError::SizeLimitExceeded { limit });
        }
        out.extend_from_slice(&buf[..n]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_bounded_accepts_below_limit() {
        let data = [42u8;100];
        let out = read_bounded(&data[..], 200).expect("ok");
        assert_eq!(out.len(), 100);
    }

    #[test]
    fn read_bounded_rejects_above_limit() {
        let data = [42u8;300];
        let err = read_bounded(&data[..], 200).unwrap_err();
        assert!(matches!(err, ThemeError::SizeLimitExceeded { limit: 200 }));
    }

    #[test]
    fn import_from_mocked_server_saves_bytes() {
        let fixture = std::fs::read("tests/fixtures/dracula.itermcolors").unwrap();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/dracula.itermcolors")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(&fixture)
            .create();

        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let url = format!("{}/dracula.itermcolors", server.url());
        let path = import_from_url(&url, "dracula-imported").expect("import");
        assert!(path.exists());

        let loaded = crate::storage::load("dracula-imported").expect("load");
        assert_eq!(loaded.background, crate::Color::new(40, 42, 54));

        std::env::remove_var("XDG_CONFIG_HOME");
        mock.assert();
    }

    #[test]
    fn import_rejects_unexpected_content_type() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/bad")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html>")
            .create();

        let tmp = tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path());

        let url = format!("{}/bad", server.url());
        let err = import_from_url(&url, "bad").unwrap_err();
        assert!(matches!(err, ThemeError::UnexpectedContentType(_)));

        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
