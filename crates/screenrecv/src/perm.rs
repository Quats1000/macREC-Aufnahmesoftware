//! macOS: Bildschirmaufnahme benoetigt eine ausdrueckliche Freigabe.

#[cfg(target_os = "macos")]
mod platform {
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }

    /// Wurde die Bildschirmaufnahme bereits erlaubt?
    pub fn preflight() -> bool {
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    /// Fordert die Freigabe an (zeigt den Systemdialog).
    pub fn request() -> bool {
        unsafe { CGRequestScreenCaptureAccess() }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    /// Unter Windows/Linux ist keine zusaetzliche Freigabe noetig.
    pub fn preflight() -> bool {
        true
    }

    pub fn request() -> bool {
        true
    }
}

pub use platform::{preflight, request};

/// Oeffnet die passende Seite der Systemeinstellungen (macOS).
pub fn open_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
            .spawn();
    }
}

/// Ist auf diesem System ueberhaupt eine Freigabe noetig?
pub fn required() -> bool {
    cfg!(target_os = "macos")
}
