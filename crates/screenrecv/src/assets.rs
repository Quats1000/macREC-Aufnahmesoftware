//! Eingebettete Ressourcen (Logo, Schriften).
//!
//! Alles wird per `include_bytes!` in die Binaerdatei gelegt, damit das
//! Programm ohne weitere Dateien vom USB-Stick startet und offline arbeitet.

pub static APP_ICON_PNG: &[u8] = include_bytes!("../resources/app-icon.png");

/// legl3r.com-Logo fuer den Startbildschirm.
pub static LOGO_PNG: &[u8] = include_bytes!("../resources/logo.png");

pub static DM_SANS_REGULAR: &[u8] = include_bytes!("../resources/fonts/DMSans-Regular.ttf");
pub static DM_SANS_MEDIUM: &[u8] = include_bytes!("../resources/fonts/DMSans-Medium.ttf");
pub static DM_SANS_BOLD: &[u8] = include_bytes!("../resources/fonts/DMSans-Bold.ttf");

pub const COPYRIGHT: &str = "(C) 2026 Niklas Legler - niklaslegler.at";

/// Link zur Unterstuetzung des Projekts.
pub const COFFEE_URL: &str = "https://buymeacoffee.com/niklaslegler";
pub const APP_TITLE: &str = "macREC - Aufnahmesoftware";
pub const BUNDLE_ID: &str = "com.legl3r.screenrecv1";
