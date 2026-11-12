//! Ablage der Aufnahmen auf dem USB-Stick.
//!
//! Zielstruktur:
//! ```text
//! <Laufwerkswurzel>/<Name>/screens/image_0001.jpg
//! <Laufwerkswurzel>/<Name>/info.json
//! <Laufwerkswurzel>/<Name>/log.jsonl
//! ```

use std::fs;
use std::path::{Path, PathBuf};

pub const SCREENS_DIR: &str = "screens";
pub const INFO_FILE: &str = "info.json";
pub const LOG_FILE: &str = "log.jsonl";

#[derive(Debug, Clone)]
pub struct Storage {
    /// Wurzel des Laufwerks (z. B. `/Volumes/PRUEFUNG` oder `E:\`).
    pub root: PathBuf,
    /// `<root>/<Name>`
    pub exam_dir: PathBuf,
    /// `<root>/<Name>/screens`
    pub screens_dir: PathBuf,
}

/// Entfernt Zeichen, die in Dateinamen nicht erlaubt sind.
pub fn sanitize_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim().to_owned();
    if trimmed.is_empty() {
        "Unbekannt".to_owned()
    } else {
        trimmed
    }
}

/// Ermittelt die Wurzel des Laufwerks, auf dem das Programm liegt.
fn volume_root(exe: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        // Der Mount-Punkt liegt direkt unter /Volumes, z. B. /Volumes/PRUEFUNG.
        for ancestor in exe.ancestors() {
            let parent = ancestor.parent();
            if parent.is_some_and(|parent| parent == Path::new("/Volumes")) {
                return Some(ancestor.to_path_buf());
            }
        }
        None
    }
    #[cfg(windows)]
    {
        // Laufwerkswurzel, z. B. "E:\\"
        use std::path::Component;
        let mut root = PathBuf::new();
        for component in exe.components() {
            if let Component::Prefix(prefix) = component {
                root.push(prefix.as_os_str());
                root.push("\\");
                return Some(root);
            }
            break;
        }
        None
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = exe;
        None
    }
}

/// Prueft, ob in ein Verzeichnis geschrieben werden kann.
fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".screenrecv-probe-{}", std::process::id()));
    match fs::create_dir_all(&probe) {
        Ok(()) => {
            let _ = fs::remove_dir(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Sucht die erste schreibbare Basis: Volume-Wurzel, dann Programmordner.
pub fn detect_root() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(vol) = volume_root(&exe) {
        candidates.push(vol);
    }
    candidates.push(exe_dir.clone());
    if let Some(parent) = exe_dir.parent() {
        candidates.push(parent.to_path_buf());
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }

    for candidate in &candidates {
        if is_writable(candidate) {
            return candidate.clone();
        }
    }
    exe_dir
}

/// Legt die Ablagestruktur an und liefert die Pfade zurueck.
pub fn prepare(root: &Path, name: &str) -> std::io::Result<Storage> {
    let exam_dir = root.join(sanitize_name(name));
    let screens_dir = exam_dir.join(SCREENS_DIR);
    fs::create_dir_all(&screens_dir)?;
    Ok(Storage {
        root: root.to_path_buf(),
        exam_dir,
        screens_dir,
    })
}

/// Naechster freier Bild-Index (`image_0001.jpg`, `image_0002.jpg`, ...).
pub fn next_image_index(screens_dir: &Path) -> u32 {
    let mut max = 0u32;
    if let Ok(entries) = fs::read_dir(screens_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else { continue };
            let Some(rest) = name.strip_prefix("image_") else { continue };
            let Some(number) = rest.strip_suffix(".jpg") else { continue };
            if let Ok(value) = number.parse::<u32>() {
                max = max.max(value);
            }
        }
    }
    max + 1
}

pub fn image_path(screens_dir: &Path, index: u32) -> PathBuf {
    screens_dir.join(format!("image_{index:04}.jpg"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_sanitized() {
        assert_eq!(sanitize_name("Max/Mustermann"), "Max_Mustermann");
        assert_eq!(sanitize_name("  "), "Unbekannt");
        assert_eq!(sanitize_name("Anna: Maria"), "Anna_ Maria");
        assert_eq!(sanitize_name(""), "Unbekannt");
    }

    #[test]
    fn image_index_continues_after_existing_files() {
        let dir = std::env::temp_dir().join("screenrecv-test-index");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(next_image_index(&dir), 1);
        std::fs::write(image_path(&dir, 1), b"x").unwrap();
        std::fs::write(image_path(&dir, 2), b"x").unwrap();
        std::fs::write(dir.join("image_0003.jpg"), b"x").unwrap();
        std::fs::write(dir.join("anderes.jpg"), b"x").unwrap();
        assert_eq!(next_image_index(&dir), 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn folders_are_created_on_the_target() {
        let root = std::env::temp_dir().join("screenrecv-test-dirs");
        let _ = std::fs::remove_dir_all(&root);
        let storage = prepare(&root, "Max/Mustermann").unwrap();
        assert!(storage.screens_dir.is_dir());
        assert!(storage.exam_dir.ends_with("Max_Mustermann"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
