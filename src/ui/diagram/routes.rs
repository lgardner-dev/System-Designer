use super::anchors::ResolvedAnchor;
use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};
/// Drawing, picking, arrowheads and animation all use this SAME sampled path.
#[derive(Clone)]
pub(in crate::ui) struct Path {
    pub points: Vec<Pos2>,
    distances: Vec<f32>,
    pub length: f32,
    pub bounds: Rect,
}
impl Path {
    pub fn new(points: Vec<Pos2>) -> Self {
        let mut distances = Vec::with_capacity(points.len());
        let mut length = 0.0;
        let mut bounds = Rect::NOTHING;
        for (i, point) in points.iter().enumerate() {
            if i > 0 {
                length += point.distance(points[i - 1]);
            }
            distances.push(length);
            bounds = bounds.union(Rect::from_min_max(*point, *point));
        }
        Self {
            points,
            distances,
            length,
            bounds,
        }
    }
    pub fn between(a: Pos2, a_normal: Vec2, b: Pos2, b_normal: Vec2) -> Self {
        let distance = (a.distance(b) * 0.4).clamp(20.0, 180.0);
        let controls = [a, a + a_normal * distance, b + b_normal * distance, b];
        let samples = (a.distance(b) / 12.0).ceil().clamp(24.0, 128.0) as usize;
        Self::sample_cubic(controls, samples)
    }
    pub fn cubic(controls: [Pos2; 4]) -> Self {
        let length: f32 = controls.windows(2).map(|w| w[0].distance(w[1])).sum();
        Self::sample_cubic(controls, (length / 12.0).ceil().clamp(24.0, 256.0) as usize)
    }
    fn sample_cubic(controls: [Pos2; 4], samples: usize) -> Self {
        Self::new(
            (0..=samples)
                .map(|i| {
                    let t = i as f32 / samples as f32;
                    let u = 1.0 - t;
                    let v = controls[0].to_vec2() * (u * u * u)
                        + controls[1].to_vec2() * (3.0 * u * u * t)
                        + controls[2].to_vec2() * (3.0 * u * t * t)
                        + controls[3].to_vec2() * (t * t * t);
                    pos2(v.x, v.y)
                })
                .collect(),
        )
    }
    pub fn at(&self, distance: f32) -> (Pos2, Vec2) {
        if self.points.len() < 2 {
            return (
                self.points.first().copied().unwrap_or(Pos2::ZERO),
                Vec2::ZERO,
            );
        }
        let distance = distance.clamp(0.0, self.length);
        let i = self
            .distances
            .partition_point(|d| *d < distance)
            .clamp(1, self.points.len() - 1);
        let a = self.points[i - 1];
        let delta = self.points[i] - a;
        let span = self.distances[i] - self.distances[i - 1];
        let t = if span > 0.0 {
            (distance - self.distances[i - 1]) / span
        } else {
            0.0
        };
        (
            a + delta * t,
            if delta.length_sq() > 0.0 {
                delta.normalized()
            } else {
                Vec2::ZERO
            },
        )
    }
    pub fn distance(&self, point: Pos2) -> f32 {
        self.points
            .windows(2)
            .map(|pair| {
                let delta = pair[1] - pair[0];
                let t = if delta.length_sq() > 0.0 {
                    ((point - pair[0]).dot(delta) / delta.length_sq()).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                point.distance(pair[0] + delta * t)
            })
            .fold(f32::INFINITY, f32::min)
    }
    pub fn screen(&self, origin: Pos2, pan: Vec2, zoom: f32) -> Self {
        Self::new(
            self.points
                .iter()
                .map(|p| {
                    super::viewport::Transform {
                        area: Rect::from_min_size(origin, Vec2::ZERO),
                        pan,
                        zoom,
                    }
                    .screen(*p)
                })
                .collect(),
        )
    }
}

/// Lane identity/order is supplied by the adapter, sorted by exact edge identity.
#[derive(Clone, Copy)]
pub(in crate::ui) struct Lane {
    pub index: usize,
    pub count: usize,
    /// Canonical unordered owner-pair axis; reversed edges keep the same lane side.
    pub axis: Vec2,
    pub loop_rect: Option<Rect>,
}
impl Default for Lane {
    fn default() -> Self {
        Self {
            index: 0,
            count: 1,
            axis: Vec2::X,
            loop_rect: None,
        }
    }
}
pub(in crate::ui) fn route(a: ResolvedAnchor, b: ResolvedAnchor, lane: Lane) -> Path {
    if let Some(rect) = lane.loop_rect {
        let reach = rect.width().max(rect.height()) * 0.8 + 80.0 * lane.index as f32;
        // Two cubics keep repeated loops separated between their single shared handles.
        let middle = rect.center()
            + if (a.normal + b.normal).length_sq() > 0.001 {
                (a.normal + b.normal).normalized()
            } else {
                a.side.normal()
            } * (reach + rect.width());
        let middle = if middle.is_finite() {
            middle
        } else {
            rect.center_top() - vec2(0.0, reach)
        };
        let tangent = vec2(-a.normal.y, a.normal.x);
        return joined(a, b, middle, tangent, reach);
    }
    if lane.count <= 1 {
        return Path::between(a.point, a.normal, b.point, b.normal);
    }
    let axis = if lane.axis.length_sq() < 0.001 {
        Vec2::X
    } else {
        lane.axis.normalized()
    };
    let side = vec2(-axis.y, axis.x);
    let offset = (lane.index as f32 - (lane.count - 1) as f32 * 0.5) * 100.0;
    let middle = a.point.lerp(b.point, 0.5) + side * offset;
    let forward = if (b.point - a.point).dot(axis) >= 0.0 {
        axis
    } else {
        -axis
    };
    joined(
        a,
        b,
        middle,
        forward,
        (a.point.distance(b.point) * 0.2).clamp(16.0, 90.0),
    )
}
fn joined(a: ResolvedAnchor, b: ResolvedAnchor, middle: Pos2, tangent: Vec2, reach: f32) -> Path {
    let mut points = Path::cubic([
        a.point,
        a.point + a.normal * reach,
        middle - tangent * reach,
        middle,
    ])
    .points;
    points.pop();
    points.extend(
        Path::cubic([
            middle,
            middle + tangent * reach,
            b.point + b.normal * reach,
            b.point,
        ])
        .points,
    );
    Path::new(points)
}
