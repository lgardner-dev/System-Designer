//! Presentation-only geometry. Port identity, direction and contracts never change.
use crate::model::{Direction, Endpoint, Node, Port, Position, Project, System, node_height};
pub(in crate::ui) use crate::ui::diagram::{anchors::Side, routes::Path};
use crate::ui::diagram::{
    anchors::{ResolvedAnchor, choose_side},
    routes,
};
use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};
use std::collections::BTreeMap;

const WIDTH: f32 = 300.0;
const ROW: f32 = 38.0;
const END_LABEL: f32 = 44.0;

fn fallback(direction: Direction) -> Side {
    if direction == Direction::In {
        Side::Left
    } else {
        Side::Right
    }
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
        pos2(
            (extent.left() - 240.0).min(40.0),
            (extent.top() - 100.0).min(40.0),
        ),
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
impl Anchor {
    pub fn resolved(&self) -> ResolvedAnchor {
        ResolvedAnchor {
            point: self.point,
            normal: self.normal,
            side: self.side,
        }
    }
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
        if let Some(old) = self.frame {
            let frame = frame_for(
                self.cards.iter().map(|c| c.rect),
                self.ports.iter().filter(|a| a.boundary).count(),
            );
            for port in self.ports.iter_mut().filter(|a| a.boundary) {
                let t = if port.side.horizontal() {
                    (port.point.x - old.left()) / old.width()
                } else {
                    (port.point.y - old.top()) / old.height()
                };
                let mut point = port.side.midpoint(frame);
                if port.side.horizontal() {
                    point.x = frame.left() + t * frame.width();
                } else {
                    point.y = frame.top() + t * frame.height();
                }
                port.label_rect = port.label_rect.translate(point - port.point);
                port.point = point;
            }
            self.frame = Some(frame);
        }
        self.bounds = self
            .cards
            .iter()
            .fold(self.frame.unwrap_or(Rect::NOTHING), |r, c| r.union(c.rect))
            .expand(50.0);
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
                return Some(routes::route(
                    a.resolved(),
                    b.resolved(),
                    routes::Lane {
                        index: lane,
                        count: lane + 1,
                        axis: Vec2::X,
                        loop_rect: Some(rect),
                    },
                ));
            }
        }
        Some(Path::between(a.point, a.normal, b.point, b.normal))
    }
}
