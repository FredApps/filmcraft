//! Plain FilmCraft community text used where the community mark would otherwise appear.

/// Kept as a compatibility shim for callers that used to request an embedded wordmark texture.
/// No brand artwork is bundled, so there is no texture to return.
pub fn wordmark(_ctx: &egui::Context, _dark: bool) -> Option<egui::TextureHandle> {
    None
}

/// Draw the community name `height` points tall with its left edge at `left_center` (vertically
/// centred). Returns the drawn rect.
pub fn paint_wordmark(ui: &egui::Ui, left_center: egui::Pos2, height: f32, dark: bool) -> Option<egui::Rect> {
    let color = if dark { egui::Color32::WHITE } else { egui::Color32::from_rgb(35, 35, 35) };
    let galley = ui.painter().layout_no_wrap("FilmCraft".to_owned(), egui::FontId::proportional(height), color);
    let rect = egui::Rect::from_min_size(egui::pos2(left_center.x, left_center.y - galley.size().y / 2.0), galley.size());
    ui.painter().galley(rect.min, galley, color);
    Some(rect)
}
