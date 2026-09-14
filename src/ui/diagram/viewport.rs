use eframe::egui::{Pos2, Rect, Vec2};
/// Shared invertible affine transform. View changes never edit document layout.
#[derive(Clone, Copy)]
pub(in crate::ui) struct Transform {
    pub area: Rect,
    pub pan: Vec2,
    pub zoom: f32,
}
impl Transform {
    pub fn screen(self, point: Pos2) -> Pos2 {
        self.area.min + (point.to_vec2() + self.pan / self.zoom) * self.zoom
    }
    pub fn world(self, point: Pos2) -> Pos2 {
        ((point - self.area.min) / self.zoom - self.pan / self.zoom).to_pos2()
    }
    pub fn rect(self, rect: Rect) -> Rect {
        Rect::from_min_max(self.screen(rect.min), self.screen(rect.max))
    }
    pub fn fit(&mut self, bounds: Rect) {
        self.zoom = ((self.area.width() - 32.0) / bounds.width())
            .min((self.area.height() - 32.0) / bounds.height())
            .clamp(0.2, 1.0);
        self.pan = self.area.size() * 0.5 - bounds.center().to_vec2() * self.zoom;
    }
    pub fn zoom_at(&mut self, cursor: Pos2, scroll: f32) {
        let world = self.world(cursor);
        self.zoom = (self.zoom * (scroll * 0.002).exp()).clamp(0.2, 2.5);
        self.pan = cursor - self.area.min - world.to_vec2() * self.zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2};
    #[test]
    fn signed_roundtrips_and_cursor_centered_zoom() {
        for zoom in [0.2, 1.0, 2.5] {
            for pan in [vec2(-1400.0, 850.0), vec2(0.0, 0.0), vec2(135.0, -410.0)] {
                let mut camera = Transform {
                    area: Rect::from_min_size(pos2(200.0, 90.0), vec2(700.0, 560.0)),
                    pan,
                    zoom,
                };
                for point in [
                    pos2(-900_000.0, 900_000.0),
                    pos2(-301.125, -701.25),
                    pos2(0.0, 0.0),
                    pos2(450.25, 730.5),
                ] {
                    let screen = camera.screen(point);
                    assert!(camera.world(screen).distance(point) < 0.2);
                    assert!(camera.screen(camera.world(screen)).distance(screen) <= 0.5);
                }
                let cursor = pos2(460.0, 370.0);
                let before = camera.world(cursor);
                camera.zoom_at(cursor, 135.0);
                assert!(camera.world(cursor).distance(before) < 0.01);
            }
        }
    }
}
