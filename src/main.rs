#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use eframe::egui;
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("System Designer")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 650.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    let initial = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    eframe::run_native(
        "System Designer",
        options,
        Box::new(move |cc| Ok(Box::new(system_designer::ui::Designer::new(cc, initial)))),
    )
}
