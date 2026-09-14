use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ui) enum Side {
    Top,
    Right,
    Bottom,
    Left,
}
impl Side {
    pub const ALL: [Self; 4] = [Self::Top, Self::Right, Self::Bottom, Self::Left];
    pub fn normal(self) -> Vec2 {
        match self {
            Self::Top => vec2(0.0, -1.0),
            Self::Right => vec2(1.0, 0.0),
            Self::Bottom => vec2(0.0, 1.0),
            Self::Left => vec2(-1.0, 0.0),
        }
    }
    pub fn midpoint(self, r: Rect) -> Pos2 {
        match self {
            Self::Top => pos2(r.center().x, r.top()),
            Self::Right => pos2(r.right(), r.center().y),
            Self::Bottom => pos2(r.center().x, r.bottom()),
            Self::Left => pos2(r.left(), r.center().y),
        }
    }
    pub fn horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
    pub fn projection(self, p: Pos2) -> f32 {
        if self.horizontal() { p.x } else { p.y }
    }
}
/// A shared/fan-out port gets ONE side, scored against all of its neighbours.
/// The normal penalty discourages initially pointing away from those neighbours.
pub fn choose_side(rect: Rect, peers: &[Pos2], default: Side, boundary: bool) -> Side {
    if peers.is_empty() {
        return default;
    }
    let cost = |side: Side| {
        let origin = side.midpoint(rect);
        let normal = side.normal() * if boundary { -1.0 } else { 1.0 };
        peers
            .iter()
            .map(|p| {
                let delta = *p - origin;
                delta.length() + 2.0 * (-delta.dot(normal)).max(0.0)
            })
            .sum::<f32>()
    };
    let mut best = default;
    for side in Side::ALL {
        if cost(side) + 0.01 < cost(best) {
            best = side;
        }
    }
    best
}

#[derive(Clone, Copy, Debug)]
pub(in crate::ui) struct ResolvedAnchor {
    pub point: Pos2,
    /// Outward normal, or inward normal for an enclosing boundary.
    pub normal: Vec2,
    pub side: Side,
}
#[derive(Clone, Copy)]
pub(in crate::ui) enum Outline {
    Rectangle,
    Diamond,
    Ellipse,
}
impl Outline {
    pub fn perimeter(self, rect: Rect, toward: Pos2) -> (Pos2, Vec2) {
        let d = toward - rect.center();
        let d = if d.length_sq() < 0.001 { Vec2::X } else { d };
        let a = rect.width() * 0.5;
        let b = rect.height() * 0.5;
        let t = match self {
            Self::Diamond => 1.0 / (d.x.abs() / a + d.y.abs() / b),
            Self::Ellipse => 1.0 / ((d.x / a).powi(2) + (d.y / b).powi(2)).sqrt(),
            Self::Rectangle => 1.0 / (d.x.abs() / a).max(d.y.abs() / b),
        };
        let v = d * t;
        let normal = match self {
            Self::Diamond => vec2(d.x.signum() / a, d.y.signum() / b).normalized(),
            Self::Ellipse => vec2(v.x / (a * a), v.y / (b * b)).normalized(),
            Self::Rectangle => {
                if (v.x.abs() - a).abs() < 0.01 {
                    vec2(d.x.signum(), 0.0)
                } else {
                    vec2(0.0, d.y.signum())
                }
            }
        };
        (rect.center() + v, normal)
    }
    pub fn anchor(self, rect: Rect, side: Side, fraction: f32) -> ResolvedAnchor {
        let target = if side.horizontal() {
            pos2(rect.left() + rect.width() * fraction, side.midpoint(rect).y)
        } else {
            pos2(side.midpoint(rect).x, rect.top() + rect.height() * fraction)
        };
        let (point, normal) = self.perimeter(rect, target);
        ResolvedAnchor {
            point,
            normal,
            side,
        }
    }
}
