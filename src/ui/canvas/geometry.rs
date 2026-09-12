//! Presentation-only geometry. Port identity, direction and contracts never change.
use crate::model::{Direction, Endpoint, Node, Port, Position, Project, System, node_height};
use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};
use std::collections::BTreeMap;

const WIDTH: f32 = 300.0;
const ROW: f32 = 38.0;
const END_LABEL: f32 = 44.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Side {
    Top,
    Right,
    Bottom,
    Left,
}
impl Side {
    const ALL: [Self; 4] = [Self::Top, Self::Right, Self::Bottom, Self::Left];
    pub fn normal(self) -> Vec2 {
        match self {
            Self::Top => vec2(0.0, -1.0),
            Self::Right => vec2(1.0, 0.0),
            Self::Bottom => vec2(0.0, 1.0),
            Self::Left => vec2(-1.0, 0.0),
        }
    }
    fn midpoint(self, r: Rect) -> Pos2 {
        match self {
            Self::Top => pos2(r.center().x, r.top()),
            Self::Right => pos2(r.right(), r.center().y),
            Self::Bottom => pos2(r.center().x, r.bottom()),
            Self::Left => pos2(r.left(), r.center().y),
        }
    }
    fn horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
    fn projection(self, p: Pos2) -> f32 {
        if self.horizontal() { p.x } else { p.y }
    }
    fn escape(self, r: Rect, p: Pos2) -> Pos2 {
        match self {
            Self::Top => pos2(p.x, r.top()),
            Self::Right => pos2(r.right(), p.y),
            Self::Bottom => pos2(p.x, r.bottom()),
            Self::Left => pos2(r.left(), p.y),
        }
    }
}
fn fallback(direction: Direction) -> Side {
    if direction == Direction::In {
        Side::Left
    } else {
        Side::Right
    }
}

/// A shared/fan-out port gets ONE side, scored against all of its neighbours.
/// The normal penalty discourages initially pointing away from those neighbours.
fn choose_side(rect: Rect, peers: &[Pos2], default: Side, boundary: bool) -> Side {
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

#[derive(Clone)]
pub(super) struct Card {
    pub id: String,
    pub rect: Rect,
    pub position: Position,
    pub header_y: f32,
}
#[derive(Clone)]
pub(super) struct Anchor {
    pub endpoint: Endpoint,
    pub point: Pos2,
    /// Points into the region in which the wire is allowed to depart.
    pub normal: Vec2,
    pub side: Side,
    pub direction: Direction,
    pub name: String,
    pub contract: String,
    pub external: String,
    pub boundary: bool,
    pub label_rect: Rect,
}
#[derive(Clone)]
pub(super) struct Scene {
    pub cards: Vec<Card>,
    pub ports: Vec<Anchor>,
    pub frame: Option<Rect>,
    pub bounds: Rect,
}
struct Placement<'a> {
    port: &'a Port,
    side: Side,
    order: f32,
    channel: String,
}
fn frame_for(rects: impl Iterator<Item = Rect>, port_count: usize) -> Rect {
    let mut extent = Rect::from_min_max(pos2(40.0, 40.0), pos2(900.0, 500.0));
    for r in rects {
        extent = extent.union(r);
    }
    Rect::from_min_max(
        pos2(40.0, 40.0),
        pos2(
            (extent.right() + 240.0).max(160.0 + port_count as f32 * 90.0),
            (extent.bottom() + 100.0).max(180.0 + port_count as f32 * 70.0),
        ),
    )
}
fn peers_for(
    system: &System,
    endpoint: &Endpoint,
    rects: &BTreeMap<String, Rect>,
    boundary: &BTreeMap<String, Pos2>,
) -> Vec<Pos2> {
    system
        .edges
        .iter()
        .filter_map(|edge| {
            let remote = if edge.from == *endpoint {
                &edge.to
            } else if edge.to == *endpoint {
                &edge.from
            } else {
                return None;
            };
            // Self-loops use an external perimeter route, not a target at our centre.
            if remote.node == endpoint.node {
                return None;
            }
            match &remote.node {
                Some(id) => rects.get(id).map(Rect::center),
                None => boundary.get(&remote.port).copied(),
            }
        })
        .collect()
}
fn placements<'a>(
    system: &System,
    owner: Option<&str>,
    ports: &'a [Port],
    rect: Rect,
    rects: &BTreeMap<String, Rect>,
    boundaries: &BTreeMap<String, Pos2>,
) -> Vec<Placement<'a>> {
    ports
        .iter()
        .map(|port| {
            let endpoint = Endpoint {
                node: owner.map(str::to_owned),
                port: port.id.clone(),
            };
            let peers = peers_for(system, &endpoint, rects, boundaries);
            let self_loop = owner.is_some()
                && system.edges.iter().any(|e| {
                    e.from.node == endpoint.node
                        && e.to.node == endpoint.node
                        && (e.from == endpoint || e.to == endpoint)
                });
            let default = if self_loop {
                Side::Top
            } else {
                fallback(port.direction)
            };
            let side = choose_side(rect, &peers, default, owner.is_none());
            let order = if peers.is_empty() {
                side.projection(rect.center())
            } else {
                peers.iter().map(|p| side.projection(*p)).sum::<f32>() / peers.len() as f32
            };
            // Equal neighbour positions should align both ends of a wire,
            // not sort inputs before outputs and cross reciprocal links.
            let channel = system
                .edges
                .iter()
                .filter(|edge| edge.from == endpoint || edge.to == endpoint)
                .map(|edge| edge.id.clone())
                .min()
                .unwrap_or_else(|| port.id.clone());
            Placement {
                port,
                side,
                order,
                channel,
            }
        })
        .collect()
}
fn count(ports: &[Placement<'_>], side: Side) -> usize {
    ports.iter().filter(|p| p.side == side).count()
}
fn card(node: &Node, position: Position, ports: &[Placement<'_>]) -> Card {
    let top = count(ports, Side::Top);
    let bottom = count(ports, Side::Bottom);
    let rows = count(ports, Side::Left)
        .max(count(ports, Side::Right))
        .max(1);
    let width = WIDTH.max((top.max(bottom) + 1) as f32 * 72.0);
    let top_space = if top > 0 { END_LABEL } else { 0.0 };
    let bottom_space = if bottom > 0 { END_LABEL } else { 0.0 };
    let origin = pos2(position.x as f32, position.y as f32);
    Card {
        id: node.id.clone(),
        position,
        header_y: origin.y + top_space,
        rect: Rect::from_min_size(
            origin,
            vec2(width, top_space + 92.0 + rows as f32 * ROW + bottom_space),
        ),
    }
}
fn anchors(
    owner: Option<&str>,
    rect: Rect,
    header_y: f32,
    planned: &[Placement<'_>],
) -> Vec<Anchor> {
    let boundary = owner.is_none();
    let mut result = Vec::with_capacity(planned.len());
    for side in Side::ALL {
        let mut group: Vec<_> = planned.iter().filter(|p| p.side == side).collect();
        group.sort_by(|a, b| {
            a.order
                .total_cmp(&b.order)
                .then(a.channel.cmp(&b.channel))
                .then(a.port.id.cmp(&b.port.id))
        });
        let horizontal_step = rect.width() / (group.len() + 1) as f32;
        for (i, planned) in group.iter().enumerate() {
            let port = planned.port;
            let point = if side.horizontal() {
                pos2(
                    rect.left() + horizontal_step * (i + 1) as f32,
                    if side == Side::Top {
                        rect.top()
                    } else {
                        rect.bottom()
                    },
                )
            } else {
                let y = if boundary {
                    rect.top() + 100.0 + i as f32 * 70.0
                } else {
                    header_y + 88.0 + i as f32 * ROW
                };
                pos2(
                    if side == Side::Left {
                        rect.left()
                    } else {
                        rect.right()
                    },
                    y,
                )
            };
            let label_width = if side.horizontal() {
                (horizontal_step - 8.0).max(20.0)
            } else if boundary {
                200.0
            } else {
                rect.width() * 0.5 - 24.0
            };
            let label_height = if boundary { 51.0 } else { 30.0 };
            let label_origin = match side {
                Side::Top => point + vec2(-label_width * 0.5, 12.0),
                Side::Bottom => point + vec2(-label_width * 0.5, -label_height - 12.0),
                Side::Left => point + vec2(14.0, -9.0),
                Side::Right => point + vec2(-label_width - 14.0, -9.0),
            };
            result.push(Anchor {
                endpoint: Endpoint {
                    node: owner.map(str::to_owned),
                    port: port.id.clone(),
                },
                point,
                normal: side.normal() * if boundary { -1.0 } else { 1.0 },
                side,
                direction: if boundary {
                    port.direction.opposite()
                } else {
                    port.direction
                },
                name: port.name.clone(),
                contract: port
                    .contract
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "Unassigned".into()),
                external: String::new(),
                boundary,
                label_rect: Rect::from_min_size(label_origin, vec2(label_width, label_height)),
            });
        }
    }
    result
}
impl Scene {
    pub fn move_card(&mut self, id: &str, position: Position) {
        if let Some(card) = self.cards.iter_mut().find(|c| c.id == id) {
            let delta = vec2(
                (position.x - card.position.x) as f32,
                (position.y - card.position.y) as f32,
            );
            card.position = position;
            card.rect = card.rect.translate(delta);
            card.header_y += delta.y;
            for port in self
                .ports
                .iter_mut()
                .filter(|p| p.endpoint.node.as_deref() == Some(id))
            {
                port.point += delta;
                port.label_rect = port.label_rect.translate(delta);
            }
        }
    }

    pub fn new(project: &Project, sid: &str, positions: &BTreeMap<String, Position>) -> Self {
        let mut result = Self {
            cards: vec![],
            ports: vec![],
            frame: None,
            bounds: Rect::NOTHING,
        };
        let Some(system) = project.system(sid) else {
            return result;
        };
        let initial: BTreeMap<_, _> = system
            .nodes
            .iter()
            .map(|n| {
                let p = positions.get(&n.id).copied().unwrap_or_default();
                (
                    n.id.clone(),
                    Rect::from_min_size(
                        pos2(p.x as f32, p.y as f32),
                        vec2(WIDTH, node_height(n) as f32),
                    ),
                )
            })
            .collect();
        let owner = project.owner(sid).map(|(_, n)| n);
        let frame = frame_for(initial.values().copied(), project.boundary(sid).len());
        let empty = BTreeMap::new();
        let boundary_plan =
            placements(system, None, project.boundary(sid), frame, &initial, &empty);
        let boundary_points: BTreeMap<_, _> = boundary_plan
            .iter()
            .map(|p| (p.port.id.clone(), p.side.midpoint(frame)))
            .collect();
        for node in &system.nodes {
            let planned = placements(
                system,
                Some(&node.id),
                &node.ports,
                initial[&node.id],
                &initial,
                &boundary_points,
            );
            let card = card(
                node,
                positions.get(&node.id).copied().unwrap_or_default(),
                &planned,
            );
            result
                .ports
                .extend(anchors(Some(&node.id), card.rect, card.header_y, &planned));
            result.bounds = result.bounds.union(card.rect);
            result.cards.push(card);
        }
        if owner.is_some() {
            let rects: BTreeMap<_, _> = result
                .cards
                .iter()
                .map(|c| (c.id.clone(), c.rect))
                .collect();
            let frame = frame_for(rects.values().copied(), project.boundary(sid).len());
            let planned = placements(system, None, project.boundary(sid), frame, &rects, &empty);
            let external = project.external_connections(sid);
            for mut anchor in anchors(None, frame, frame.top(), &planned) {
                let names = external
                    .iter()
                    .find(|e| e["port"].as_str() == Some(&anchor.endpoint.port))
                    .and_then(|e| e["links"].as_array())
                    .map(|links| {
                        links
                            .iter()
                            .filter_map(|l| l["name"].as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                anchor.external = if names.is_empty() {
                    "No external connection".into()
                } else {
                    format!(
                        "{} {names}",
                        if anchor.direction == Direction::Out {
                            "from"
                        } else {
                            "to"
                        }
                    )
                };
                result.ports.push(anchor);
            }
            result.frame = Some(frame);
            result.bounds = frame;
        }
        if result.cards.is_empty() && result.frame.is_none() {
            result.bounds = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 550.0));
        }
        result.bounds = result.bounds.expand(50.0);
        result
    }
    pub fn port(&self, endpoint: &Endpoint) -> Option<&Anchor> {
        self.ports.iter().find(|p| p.endpoint == *endpoint)
    }
    pub fn route(&self, from: &Endpoint, to: &Endpoint, lane: usize) -> Option<Path> {
        let a = self.port(from)?;
        let b = self.port(to)?;
        if let Some(id) = &from.node {
            if to.node.as_ref() == Some(id) {
                let rect = self.cards.iter().find(|c| c.id == *id)?.rect;
                return Some(Path::loop_around(a, b, rect, 36.0 + lane as f32 * 9.0));
            }
        }
        Some(Path::between(a.point, a.normal, b.point, b.normal))
    }
}

/// Drawing, picking, arrowheads and animation all use this SAME sampled path.
#[derive(Clone)]
pub(super) struct Path {
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
    fn loop_around(a: &Anchor, b: &Anchor, rect: Rect, margin: f32) -> Self {
        let outer = rect.expand(margin);
        let escape_a = a.side.escape(outer, a.point);
        let escape_b = b.side.escape(outer, b.point);
        let w = outer.width();
        let h = outer.height();
        let perimeter = 2.0 * (w + h);
        let offset = |p: Pos2, side: Side| match side {
            Side::Top => p.x - outer.left(),
            Side::Right => w + p.y - outer.top(),
            Side::Bottom => w + h + outer.right() - p.x,
            Side::Left => 2.0 * w + h + outer.bottom() - p.y,
        };
        let start = offset(escape_a, a.side);
        let end = offset(escape_b, b.side);
        let clockwise = (end - start).rem_euclid(perimeter);
        let forward = clockwise <= perimeter * 0.5;
        let travel = if forward {
            clockwise
        } else {
            perimeter - clockwise
        };
        let mut corners: Vec<_> = [
            (0.0, outer.left_top()),
            (w, outer.right_top()),
            (w + h, outer.right_bottom()),
            (2.0 * w + h, outer.left_bottom()),
        ]
        .into_iter()
        .filter_map(|(at, p)| {
            let distance = if forward { at - start } else { start - at }.rem_euclid(perimeter);
            (distance > 0.01 && distance < travel - 0.01).then_some((distance, p))
        })
        .collect();
        corners.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut points = vec![a.point, escape_a];
        points.extend(corners.into_iter().map(|(_, p)| p));
        points.extend([escape_b, b.point]);
        Self::new(points)
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
                .map(|p| origin + pan + p.to_vec2() * zoom)
                .collect(),
        )
    }
}
