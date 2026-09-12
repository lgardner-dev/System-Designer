//! Embedded product artwork, independent of any user's project.
use eframe::egui::{self, ColorImage, IconData, TextureHandle, TextureOptions};
use std::sync::OnceLock;

const PNG: &[u8] = include_bytes!("../../assets/branding/linux/png/system-designer-256.png");
static ICON: OnceLock<IconData> = OnceLock::new();

/// The supplied PNG is compiled in; no file, service or image loader is required.
pub fn window_icon() -> IconData {
    ICON.get_or_init(|| {
        eframe::icon_data::from_png_bytes(PNG)
            .expect("the checked-in System Designer icon must be a valid PNG")
    })
    .clone()
}

fn texture(ctx: &egui::Context) -> TextureHandle {
    let id = egui::Id::new("system-designer.brand-icon");
    if let Some(texture) = ctx.data(|data| data.get_temp::<TextureHandle>(id)) {
        return texture;
    }
    let icon = window_icon();
    let image =
        ColorImage::from_rgba_unmultiplied([icon.width as usize, icon.height as usize], &icon.rgba);
    let texture = ctx.load_texture("System Designer atom", image, TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

pub(super) fn show(ui: &mut egui::Ui, size: f32) {
    let texture = texture(ui.ctx());
    ui.add(
        egui::Image::new((texture.id(), egui::vec2(size, size)))
            .alt_text("System Designer atom icon"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_decodes_to_complete_rgba() {
        let icon = window_icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
        assert!(icon.rgba.chunks_exact(4).any(|p| p[3] > 0));
        assert_eq!(window_icon().rgba, icon.rgba);
    }

    #[test]
    fn icon_texture_is_cached_per_context() {
        let ctx = egui::Context::default();
        assert_eq!(texture(&ctx).id(), texture(&ctx).id());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show(ui, 24.0);
                show(ui, 64.0);
            });
        });
    }
}
