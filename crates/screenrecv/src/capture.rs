//! Aufnahme des Bildschirms und Speicherung als JPEG.
//!
//! Der Aufnahme-Thread steuert den Zeitplan selbst, damit die Aufnahme auch
//! weiterlaeuft, wenn das Fenster minimiert oder geschlossen ist.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, RgbaImage};
use xcap::Monitor;

use crate::storage::Storage;
use crate::watermark;

/// Kleinster Abstand zwischen zwei Aufnahmen (Sekunden).
pub const MIN_INTERVAL: u64 = 35;
/// Groesster Abstand zwischen zwei Aufnahmen (Sekunden).
pub const MAX_INTERVAL: u64 = 60;

#[derive(Debug)]
pub enum Event {
    /// Eine Aufnahme wurde gespeichert.
    Captured {
        index: u32,
        path: PathBuf,
        bytes: u64,
        width: u32,
        height: u32,
        at: String,
    },
    /// Die Aufnahme ist fehlgeschlagen, der naechste Versuch folgt.
    Failed {
        message: String,
    },
}

/// Liefert den haupt- (bzw. ersten) Monitor.
fn primary_monitor() -> Result<Monitor, String> {
    let monitors = Monitor::all().map_err(|err| format!("Monitore nicht abrufbar: {err}"))?;
    if monitors.is_empty() {
        return Err("Kein Monitor gefunden.".to_owned());
    }
    let primary = monitors
        .iter()
        .find(|monitor| monitor.is_primary().unwrap_or(false));
    Ok(primary.unwrap_or(&monitors[0]).clone())
}

/// Nimmt den gesamten Bildschirm auf.
pub fn capture_screen() -> Result<RgbaImage, String> {
    let monitor = primary_monitor()?;
    monitor
        .capture_image()
        .map_err(|err| format!("Aufnahme nicht möglich: {err}"))
}

/// Speichert ein Bild als JPEG (Qualitaet 85).
pub fn write_jpeg(path: &PathBuf, image: &RgbaImage) -> Result<u64, String> {
    // JPEG kann kein Alpha - daher auf RGB konvertieren.
    let rgb = image::DynamicImage::ImageRgba8(image.clone()).to_rgb8();

    // Erst komplett encodieren, dann schreiben - so bleiben keine leeren
    // Dateien zurueck, wenn die Aufnahme fehlschlaegt.
    let mut buffer: Vec<u8> = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 85);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ExtendedColorType::Rgb8,
        )
        .map_err(|err| format!("JPEG-Encoder: {err}"))?;

    let file = File::create(path).map_err(|err| format!("Datei nicht schreibbar: {err}"))?;
    let mut writer = BufWriter::new(file);
    writer
        .write_all(&buffer)
        .map_err(|err| format!("Datei nicht schreibbar: {err}"))?;
    writer
        .flush()
        .map_err(|err| format!("Datei nicht schreibbar: {err}"))?;
    Ok(buffer.len() as u64)
}

/// Zufaelliger Abstand zwischen zwei Aufnahmen.
pub fn next_interval() -> Duration {
    let seconds = rand::rng().random_range(MIN_INTERVAL..=MAX_INTERVAL);
    Duration::from_secs(seconds)
}

use rand::Rng;

/// Hintergrund-Thread, der in unregelmaessigen Abstaenden aufnimmt.
pub struct Recorder {
    pub rx: Receiver<Event>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Recorder {
    pub fn start(storage: Storage) -> Recorder {
        let (evt_tx, evt_rx) = std::sync::mpsc::channel::<Event>();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);

        let handle = thread::Builder::new()
            .name("screenrecv-capture".to_owned())
            .spawn(move || {
                let mut index = crate::storage::next_image_index(&storage.screens_dir);
                loop {
                    if stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    let result = capture_screen().and_then(|raw| {
                        let stamped = watermark::apply(raw);
                        let path = crate::storage::image_path(&storage.screens_dir, index);
                        let bytes = write_jpeg(&path, &stamped)?;
                        Ok((path, bytes, stamped.width(), stamped.height()))
                    });

                    // Der Abstand wird vor der Aufnahme festgelegt, damit die
                    // Anzeige (Countdown) und der Thread dieselbe Zeit teilen.
                    let next_at = Instant::now() + next_interval();
                    match result {
                        Ok((path, bytes, width, height)) => {
                            let at =
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            if evt_tx
                                .send(Event::Captured {
                                    index,
                                    path,
                                    bytes,
                                    width,
                                    height,
                                    at,
                                })
                                .is_err()
                            {
                                break;
                            }
                            index += 1;
                        }
                        Err(err) => {
                            if evt_tx
                                .send(Event::Failed { message: err })
                                .is_err()
                            {
                                break;
                            }
                        }
                    }

                    // Warten, aber auf das Stoppsignal reagieren.
                    while Instant::now() < next_at {
                        if stop_flag.load(Ordering::Relaxed) {
                            return;
                        }
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            })
            .expect("Aufnahme-Thread starten");

        Recorder {
            rx: evt_rx,
            stop,
            thread: Some(handle),
        }
    }

    /// Beendet die Aufnahme.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}
