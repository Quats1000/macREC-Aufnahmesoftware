//! screenrecv - Pruefungsaufsicht
//!
//! Das Programm wird vom USB-Stick gestartet, fuehrt den Pruefling durch einen
//! kurzen Wizard und nimmt danach in regelmaessigen, zufaelligen Abstaenden
//! Screenshots des gesamten Bildschirms auf. Die Aufnahmen werden direkt auf
//! dem Stick unter `<Laufwerk>/<Name>/screens/` abgelegt.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod capture;
mod manifest;
mod perm;
mod storage;
mod theme;
mod watermark;

use std::path::{Path, PathBuf};

use eframe::egui;
use egui::{Align, Color32, FontId, Layout, RichText, Vec2};

use crate::capture::{Event, Recorder};
use crate::manifest::{LogEntry, Manifest};
use crate::storage::Storage;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Screen {
    Wizard,
    Permission,
    Form,
    Recording,
}

struct ScreenRecvApp {
    screen: Screen,
    logo: egui::TextureHandle,

    // Formular
    name: String,
    class: String,
    test: String,
    date: String,

    // Ablage
    root: PathBuf,
    storage: Option<Storage>,

    // Aufnahme
    recorder: Option<Recorder>,
    count: u32,
    last_file: Option<String>,
    status: Option<(String, bool)>,

    // Laufzeit
    started_at: String,
    manifest: Manifest,
    quit: bool,
    screenshot_done: bool,
}

impl ScreenRecvApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let logo = load_logo(&cc.egui_ctx);

        // Entwicklungshilfe: SCREEN=form|permission|recording startet direkt
        // auf dem jeweiligen Bildschirm (fuer Screenshots).
        let screen = match std::env::var("SCREEN").as_deref() {
            Ok("form") => Screen::Form,
            Ok("permission") => Screen::Permission,
            Ok("recording") => Screen::Recording,
            _ => Screen::Wizard,
        };

        let mut app = ScreenRecvApp {
            screen,
            logo,
            name: String::new(),
            class: String::new(),
            test: String::new(),
            date: chrono::Local::now().format("%d.%m.%Y").to_string(),
            root: storage::detect_root(),
            storage: None,
            recorder: None,
            count: if screen == Screen::Recording { 7 } else { 0 },
            last_file: None,
            status: None,
            started_at: now_string(),
            manifest: Manifest::default(),
            quit: false,
            screenshot_done: false,
        };

        // Entwicklungshilfe: AUTOSTART=1 startet die Aufnahme sofort.
        if std::env::var("AUTOSTART").is_ok() {
            app.name = "Testperson".to_owned();
            app.class = "5B".to_owned();
            app.test = "Mathematik".to_owned();
            app.start_recording();
        }

        app
    }

    // ---------------------------------------------------------------- Formular

    fn form_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.class.trim().is_empty()
            && !self.test.trim().is_empty()
    }

    fn start_recording(&mut self) {
        let prepared = match storage::prepare(&self.root, &self.name) {
            Ok(storage) => storage,
            Err(err) => {
                self.status = Some((
                    format!("Der Ordner konnte nicht angelegt werden: {err}"),
                    true,
                ));
                return;
            }
        };

        self.manifest = Manifest {
            app: assets::APP_TITLE.to_owned(),
            bundle_id: assets::BUNDLE_ID.to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            name: self.name.trim().to_owned(),
            class: self.class.trim().to_owned(),
            test: self.test.trim().to_owned(),
            date: self.date.clone(),
            started_at: self.started_at.clone(),
            ended_at: None,
            screenshots: 0,
            last_screenshot_at: None,
            storage_root: prepared.root.display().to_string(),
            device: manifest::device_name(),
        };
        if let Err(err) = self.manifest.write(&prepared.exam_dir) {
            self.status = Some((format!("info.json nicht schreibbar: {err}"), true));
        }

        self.recorder = Some(Recorder::start(prepared.clone()));
        self.storage = Some(prepared);
        self.count = 0;
        self.screen = Screen::Recording;
        self.status = None;
    }

    fn stop_recording(&mut self) {
        if let Some(mut recorder) = self.recorder.take() {
            recorder.stop();
        }
        self.finalize_manifest();
        self.quit = true;
    }

    fn finalize_manifest(&mut self) {
        if let Some(storage) = &self.storage {
            self.manifest.ended_at = Some(now_string());
            self.manifest.screenshots = self.count;
            let _ = self.manifest.write(&storage.exam_dir);
        }
    }

    // ------------------------------------------------------------------ Layout

    fn show_footer(&self, ui: &mut egui::Ui) {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(assets::COPYRIGHT)
                        .font(FontId::new(12.0, theme::sans()))
                        .color(theme::MUTED),
                );
                ui.add_space(14.0);
                ui.hyperlink_to(
                    RichText::new("Buy me a coffee")
                        .font(FontId::new(12.0, theme::sans()))
                        .color(theme::PURPLE),
                    assets::COFFEE_URL,
                );
            });
        });
    }

    fn show_wizard(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            ui.add(
                egui::Image::from_texture(&self.logo)
                    .fit_to_exact_size(Vec2::splat(56.0)),
            );
            ui.add_space(20.0);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.set_max_width(440.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            "Hallo, dieses Programm stellt sicher, dass du bei deiner Prüfung nicht schummelst.",
                        )
                        .font(FontId::new(28.0, theme::sans_bold()))
                        .color(theme::DARK)
                        .line_height(Some(36.0)),
                    )
                    .wrap(),
                );
                ui.add_space(14.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            "Während der Prüfung werden in unregelmäßigen Abständen Screenshots deines Bildschirms erstellt und direkt auf diesem USB-Stick gespeichert.",
                        )
                        .font(FontId::new(16.0, theme::sans()))
                        .color(theme::MUTED)
                        .line_height(Some(24.0)),
                    )
                    .wrap(),
                );
                ui.add_space(32.0);
                if theme::capsule(ui, "Weiter").clicked() {
                    self.screen = if perm::required() && !perm::preflight() {
                        Screen::Permission
                    } else {
                        Screen::Form
                    };
                }
            });
        });
    }

    fn show_permission(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(36.0);
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.set_max_width(460.0);
                theme::title(ui, "Bildschirmaufnahme erlauben");
                ui.add_space(10.0);
                theme::subtitle(
                    ui,
                    "macOS blockiert Bildschirmaufnahmen, bis du das Programm einmalig freigibst. Dafür brauchst du nur dein Benutzerkonto – keine Administratorrechte.",
                );
                ui.add_space(28.0);
                theme::label(ui, "So geht's");
                ui.add_space(12.0);
                for (index, step) in [
                    "Systemeinstellungen → Datenschutz & Sicherheit → Bildschirmaufnahme",
                    "Programm „macREC“ aktivieren",
                    "Zurück hierher klicken und „Erneut prüfen“ wählen",
                ]
                .iter()
                .enumerate()
                {
                    ui.horizontal_top(|ui| {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(format!("{}.", index + 1))
                                .font(FontId::new(15.0, theme::sans_bold()))
                                .color(theme::PURPLE),
                        );
                        ui.label(
                            RichText::new(*step)
                                .font(FontId::new(15.0, theme::sans()))
                                .color(theme::DARK)
                                .line_height(Some(22.0)),
                        );
                    });
                    ui.add_space(10.0);
                }
                ui.add_space(20.0);
                if theme::capsule(ui, "Erneut prüfen").clicked() {
                    if perm::request() && perm::preflight() {
                        self.screen = Screen::Form;
                    } else {
                        self.status = Some((
                            "Die Freigabe ist noch nicht erteilt. Bitte den Anweisungen folgen."
                                .to_owned(),
                            true,
                        ));
                    }
                }
                ui.add_space(12.0);
                if theme::text_button(ui, "Systemeinstellungen öffnen").clicked() {
                    perm::open_settings();
                }
                ui.add_space(6.0);
                if theme::text_button(ui, "Zurück").clicked() {
                    self.screen = Screen::Wizard;
                }
                if let Some((message, _)) = &self.status {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(message)
                            .font(FontId::new(14.0, theme::sans()))
                            .color(Color32::from_rgb(0xc0, 0x2b, 0x2b)),
                    );
                }
            });
        });
    }

    fn show_form(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.set_max_width(460.0);
                theme::title(ui, "Angaben zur Prüfung");
                ui.add_space(8.0);
                theme::subtitle(
                    ui,
                    "Diese Angaben landen gemeinsam mit den Screenshots auf dem Stick.",
                );
                ui.add_space(26.0);

                theme::label(ui, "Name");
                ui.add_space(8.0);
                theme::field(ui, &mut self.name, "Vor- und Nachname");
                ui.add_space(16.0);

                theme::label(ui, "Datum");
                ui.add_space(8.0);
                theme::read_only_field(ui, &self.date);
                ui.add_space(16.0);

                theme::label(ui, "Klasse");
                ui.add_space(8.0);
                theme::field(ui, &mut self.class, "z. B. 5B");
                ui.add_space(16.0);

                theme::label(ui, "Prüfung / Test");
                ui.add_space(8.0);
                theme::field(ui, &mut self.test, "z. B. Mathematik, 1. Teil");
                ui.add_space(26.0);

                if theme::capsule(ui, "Beginne Aufnahme").clicked() {
                    if self.form_valid() {
                        self.start_recording();
                    } else {
                        self.status = Some((
                            "Bitte Name, Klasse und Prüfung ausfüllen.".to_owned(),
                            true,
                        ));
                    }
                }
                ui.add_space(10.0);
                if theme::text_button(ui, "Zurück").clicked() {
                    self.screen = Screen::Wizard;
                }
                if let Some((message, _)) = &self.status {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(message)
                            .font(FontId::new(14.0, theme::sans()))
                            .color(Color32::from_rgb(0xc0, 0x2b, 0x2b)),
                    );
                }
            });
        });
    }

    fn show_recording(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.set_max_width(460.0);
                theme::title(ui, "Aufnahme läuft");
                ui.add_space(8.0);
                theme::subtitle(
                    ui,
                    "Du kannst das Fenster minimieren – die Aufnahme läuft im Hintergrund weiter.",
                );
                ui.add_space(30.0);

                theme::label(ui, "Screenshots");
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("{}", self.count))
                        .font(FontId::new(40.0, theme::sans_bold()))
                        .color(theme::DARK),
                );

                theme::hairline(ui);

                theme::label(ui, "Ablage");
                ui.add_space(8.0);
                ui.label(
                    RichText::new(self.storage_path())
                        .font(FontId::new(14.0, theme::sans()))
                        .color(theme::DARK),
                );
                if let Some(last) = &self.last_file {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("Zuletzt gespeichert: {last}"))
                            .font(FontId::new(13.0, theme::sans()))
                            .color(theme::MUTED),
                    );
                }

                if let Some((message, is_error)) = &self.status {
                    ui.add_space(16.0);
                    let color = if *is_error {
                        Color32::from_rgb(0xc0, 0x2b, 0x2b)
                    } else {
                        theme::MUTED
                    };
                    ui.label(
                        RichText::new(message)
                            .font(FontId::new(13.0, theme::sans()))
                            .color(color),
                    );
                }

                ui.add_space(30.0);
                if theme::soft_capsule(ui, "Aufnahme beenden").clicked() {
                    self.stop_recording();
                }
            });
        });
    }

    fn storage_path(&self) -> String {
        self.storage
            .as_ref()
            .map(|storage| storage.screens_dir.display().to_string())
            .unwrap_or_else(|| self.root.display().to_string())
    }

    // ----------------------------------------------------------------- Aufnahme

    fn poll_events(&mut self) {
        let Some(recorder) = &self.recorder else { return };
        let mut pending = Vec::new();
        while let Ok(event) = recorder.rx.try_recv() {
            pending.push(event);
        }

        for event in pending {
            match event {
                Event::Captured {
                    index,
                    path,
                    bytes,
                    width,
                    height,
                    at,
                    ..
                } => {
                    self.count = index + 1;
                    self.last_file = path
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string());
                    self.status = Some((format!("Aufnahme gespeichert ({bytes} Bytes)."), false));

                    if let Some(storage) = &self.storage {
                        self.manifest.screenshots = self.count;
                        self.manifest.last_screenshot_at = Some(at.clone());
                        let _ = self.manifest.write(&storage.exam_dir);
                        let entry = LogEntry {
                            index,
                            timestamp: at,
                            file: path
                                .file_name()
                                .map(|name| name.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            bytes,
                            width,
                            height,
                        };
                        let _ = manifest::append_log(&storage.exam_dir, &entry);
                    }
                }
                Event::Failed { message, .. } => {
                    self.status = Some((format!("Aufnahme fehlgeschlagen: {message}"), true));
                }
            }
        }
    }

    /// Verhindert, dass das Fenster geschlossen wird - die Aufnahme laeuft weiter.
    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if self.screen != Screen::Recording {
            return;
        }
        let requested = ctx.input(|input| input.viewport().close_requested());
        if requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            self.status = Some((
                "Die Aufnahme läuft weiter. Beenden erst über „Aufnahme beenden“.".to_owned(),
                false,
            ));
        }
    }
}

/// Laedt das legl3r.com-Logo als Textur (egui hat keinen eingebauten PNG-Decoder).
fn load_logo(ctx: &egui::Context) -> egui::TextureHandle {
    let options = egui::TextureOptions::LINEAR;
    let image = image::load_from_memory_with_format(assets::LOGO_PNG, image::ImageFormat::Png)
        .map(|image| image.into_rgba8());

    match image {
        Ok(image) => {
            let (width, height) = image.dimensions();
            let color = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                image.as_raw(),
            );
            ctx.load_texture("logo", color, options)
        }
        Err(_) => ctx.load_texture(
            "logo",
            egui::ColorImage::new([1, 1], vec![egui::Color32::TRANSPARENT]),
            options,
        ),
    }
}

/// Fenster-Icon (Taskbar / Dock).
fn load_icon() -> Option<egui::IconData> {
    let image = image::load_from_memory_with_format(assets::APP_ICON_PNG, image::ImageFormat::Png)
        .ok()?
        .into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn now_string() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Speichert einen Screenshot der Oberflaeche (nur fuer die Entwicklung).
fn save_ui_screenshot(path: &Path, image: &egui::ColorImage) {
    let size = [image.size[0] as u32, image.size[1] as u32];
    let mut buffer = Vec::with_capacity((size[0] * size[1] * 4) as usize);
    for pixel in &image.pixels {
        buffer.extend_from_slice(&pixel.to_array());
    }
    if let Some(rgba) = image::RgbaImage::from_raw(size[0], size[1], buffer) {
        let _ = rgba.save(path);
    }
}

impl eframe::App for ScreenRecvApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.poll_events();
        self.handle_close_request(&ctx);

        // Entwicklungs-Hilfe: Fensterinhalt als PNG sichern.
        if !self.screenshot_done {
            if let Ok(path) = std::env::var("SCREENSHOT") {
                let path = PathBuf::from(path);
                self.screenshot_done = true;
                ctx.request_screenshot(move |image| save_ui_screenshot(&path, &image));
            }
        }

        egui::Panel::bottom("footer")
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::WHITE)
                    .inner_margin(egui::Margin::symmetric(28, 16))
                    .outer_margin(egui::Margin::symmetric(28, 0)),
            )
            .show(ui, |ui| self.show_footer(ui));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::WHITE)
                    .inner_margin(egui::Margin::symmetric(28, 24)),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.screen {
                        Screen::Wizard => self.show_wizard(ui),
                        Screen::Permission => self.show_permission(ui),
                        Screen::Form => self.show_form(ui),
                        Screen::Recording => self.show_recording(ui),
                    });
            });

        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Countdown und Ereignisse aktuell halten.
        if self.screen == Screen::Recording {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }

    /// Hintergrund des Fensters: reinweiss, damit keine dunklen Randflaechen
    /// entstehen (egui/eframe nutzt sonst ein fast schwarzes Standardgrau).
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [1.0, 1.0, 1.0, 1.0]
    }

    fn on_exit(&mut self) {
        if let Some(mut recorder) = self.recorder.take() {
            recorder.stop();
        }
        self.finalize_manifest();
    }
}

/// Testmodus: eine Bildschirmaufnahme erstellen (z. B. zum Prüfen der
/// Berechtigung): `screenrecv --capture /tmp/test.jpg`
fn capture_test(target: &str) -> i32 {
    match capture::capture_screen() {
        Ok(image) => {
            let stamped = watermark::apply(image);
            let path = PathBuf::from(target);
            match capture::write_jpeg(&path, &stamped) {
                Ok(bytes) => {
                    println!(
                        "Aufnahme gespeichert: {} ({} Bytes, {}x{})",
                        path.display(),
                        bytes,
                        stamped.width(),
                        stamped.height()
                    );
                    0
                }
                Err(err) => {
                    println!("Speichern fehlgeschlagen: {err}");
                    2
                }
            }
        }
        Err(err) => {
            println!("Aufnahme fehlgeschlagen: {err}");
            1
        }
    }
}

fn main() -> eframe::Result<()> {
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(index) = arguments.iter().position(|arg| arg == "--capture") {
        let target = arguments.get(index + 1).cloned().unwrap_or_else(|| {
            std::env::temp_dir()
                .join("screenrecv-test.jpg")
                .display()
                .to_string()
        });
        std::process::exit(capture_test(&target));
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title(assets::APP_TITLE)
        .with_inner_size([660.0, 780.0])
        .with_min_inner_size([560.0, 600.0])
        .with_resizable(true);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        assets::APP_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(ScreenRecvApp::new(cc)))),
    )
}
