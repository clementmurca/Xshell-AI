#![cfg(target_os = "macos")]

use crate::error::{FinderError, Result};
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSModalResponse, NSOpenPanel};
use objc2_foundation::{MainThreadMarker, NSString, NSURL};
use std::path::{Path, PathBuf};

const NS_MODAL_RESPONSE_OK: NSModalResponse = 1;

/// Exécute un `NSOpenPanel` synchrone. DOIT être appelé depuis le main thread Cocoa.
/// Retourne `Ok(Some(path))` si l'utilisateur valide, `Ok(None)` s'il annule.
pub fn run_ns_open_panel(title: &str, default_path: Option<&Path>) -> Result<Option<PathBuf>> {
    // SAFETY: Le contrat de `run_ns_open_panel` exige d'être invoqué depuis le main thread Cocoa.
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    // Initialise NSApplication si le runtime Cocoa n'est pas déjà up.
    let _app = NSApplication::sharedApplication(mtm);

    unsafe {
        let panel: Retained<NSOpenPanel> = NSOpenPanel::openPanel(mtm);

        panel.setCanChooseDirectories(true);
        panel.setCanChooseFiles(false);
        panel.setAllowsMultipleSelection(false);
        panel.setResolvesAliases(true);
        panel.setCanCreateDirectories(true);

        let ns_title = NSString::from_str(title);
        panel.setTitle(Some(&ns_title));
        panel.setMessage(Some(&ns_title));

        if let Some(p) = default_path {
            if let Some(s) = p.to_str() {
                let url = NSURL::fileURLWithPath(&NSString::from_str(s));
                panel.setDirectoryURL(Some(&url));
            }
        }

        let response: NSModalResponse = panel.runModal();
        if response != NS_MODAL_RESPONSE_OK {
            return Ok(None);
        }

        let urls = panel.URLs();
        let Some(url) = urls.firstObject() else {
            return Ok(None);
        };

        let path = nsurl_to_pathbuf(&url)?;
        Ok(Some(path))
    }
}

/// Convertit un `NSURL` file:// en `PathBuf`. Erreur si l'URL n'est pas locale.
fn nsurl_to_pathbuf(url: &NSURL) -> Result<PathBuf> {
    unsafe {
        if !url.isFileURL() {
            return Err(FinderError::NonFileUrl);
        }
        let Some(ns_path) = url.path() else {
            return Err(FinderError::NonFileUrl);
        };
        Ok(PathBuf::from(ns_path.to_string()))
    }
}
