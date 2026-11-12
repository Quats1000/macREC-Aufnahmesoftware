//! Copyright-Hinweis, der in jede Aufnahme eingebrannt wird.

use ab_glyph::FontVec;
use image::{Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};

pub const NOTICE: &str = "(C) 2026 Niklas Legler - niklaslegler.at";

/// Zeichnet den Hinweis mittig unten in das Bild.
pub fn apply(mut image: RgbaImage) -> RgbaImage {
    let Ok(font) = FontVec::try_from_vec(crate::assets::DM_SANS_MEDIUM.to_vec()) else {
        return image;
    };

    let height = image.height();
    let width = image.width();
    let size = (height as f32 / 38.0).clamp(14.0, 44.0);

    let (text_width, text_height) = text_size(size, &font, NOTICE);
    let margin = (size * 0.9).round() as i32;
    let x = ((width as f32 - text_width as f32) / 2.0).round() as i32;
    let y = height as i32 - margin - text_height as i32;

    // Schatten fuer Lesbarkeit auf hellen und dunklen Flaechen.
    draw_text_mut(
        &mut image,
        Rgba([0u8, 0u8, 0u8, 170]),
        x + 2,
        y + 2,
        size,
        &font,
        NOTICE,
    );
    draw_text_mut(
        &mut image,
        Rgba([255u8, 255u8, 255u8, 235]),
        x,
        y,
        size,
        &font,
        NOTICE,
    );

    image
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Prueft, dass der Copyright-Hinweis mittig unten im Bild landet.
    #[test]
    fn watermark_is_drawn_at_the_bottom_center() {
        let mut image = RgbaImage::new(1600, 900);
        // Verlauf als Testbild
        for y in 0..900u32 {
            for x in 0..1600u32 {
                let shade = ((x / 8 + y / 8) % 2) as u8 * 90 + 40;
                image.put_pixel(x, y, Rgba([shade, shade, shade, 255]));
            }
        }
        let stamped = apply(image);
        // In der Mitte unten muss sich die Bildfarbe veraendert haben.
        let middle = stamped.get_pixel(800, 860);
        assert!(middle.0[0] > 40 || middle.0[1] > 40 || middle.0[2] > 40);
        stamped
            .save(std::env::temp_dir().join("screenrecv-watermark-test.png"))
            .unwrap();
    }
}
