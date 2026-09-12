//! Reversible visual summaries. Never serialized as replacement project edges.
use super::*;
use crate::model::Edge;
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui) enum View {
    #[default]
    Detail,
    Overview,
}
#[derive(Clone)]
pub(super) struct Link<'a> {
    pub edge: &'a Edge,
    pub members: Vec<String>,
    pub path: Path,
    pub label: String,
    pub selection: Selection,
}
/// Node pairs may be summarized, but boundary ports and self-loops stay exact.
/// Opposite directions are always different groups.
pub(super) fn groups(system: &System) -> Vec<Vec<&Edge>> {
    let mut grouped: BTreeMap<(String, String), Vec<&Edge>> = BTreeMap::new();
    let mut exact = vec![];
    for edge in &system.edges {
        match (&edge.from.node, &edge.to.node) {
            (Some(a), Some(b)) if a != b => grouped
                .entry((a.clone(), b.clone()))
                .or_default()
                .push(edge),
            _ => exact.push(vec![edge]),
        }
    }
    exact.extend(grouped.into_values());
    for group in &mut exact {
        group.sort_by(|a, b| a.id.cmp(&b.id));
    }
    exact.sort_by(|a, b| a[0].id.cmp(&b[0].id));
    exact
}
/// Keep node origins stable across zoom/details; collapse only their drawing bounds.
/// Project and hidden child systems are never cloned into a weaker model.
pub(super) fn compact_scene(mut scene: Scene) -> Scene {
    for card in &mut scene.cards {
        card.rect.max = card.rect.min + vec2(250.0, 112.0);
        card.header_y = card.rect.top();
        for side in [Side::Top, Side::Right, Side::Bottom, Side::Left] {
            let mut ports: Vec<_> = scene
                .ports
                .iter_mut()
                .filter(|a| a.endpoint.node.as_ref() == Some(&card.id) && a.side == side)
                .collect();
            let count = ports.len();
            for (i, a) in ports.iter_mut().enumerate() {
                let t = (i + 1) as f32 / (count + 1) as f32;
                a.point = match side {
                    Side::Top => pos2(card.rect.left() + t * card.rect.width(), card.rect.top()),
                    Side::Bottom => {
                        pos2(card.rect.left() + t * card.rect.width(), card.rect.bottom())
                    }
                    Side::Left => pos2(card.rect.left(), card.rect.top() + t * card.rect.height()),
                    Side::Right => {
                        pos2(card.rect.right(), card.rect.top() + t * card.rect.height())
                    }
                };
            }
        }
    }
    // Owner frames and their exact physical boundary ports are deliberately kept.
    if scene.frame.is_none() && !scene.cards.is_empty() {
        scene.bounds = scene
            .cards
            .iter()
            .fold(Rect::NOTHING, |r, c| r.union(c.rect))
            .expand(50.0);
    }
    scene
}
fn summary_anchor(rect: Rect, target: Pos2, offset: f32) -> (Pos2, Vec2) {
    let delta = target - rect.center();
    if (delta.x / rect.width()).abs() >= (delta.y / rect.height()).abs() {
        let normal = vec2(if delta.x >= 0.0 { 1.0 } else { -1.0 }, 0.0);
        (
            pos2(
                if normal.x > 0.0 {
                    rect.right()
                } else {
                    rect.left()
                },
                rect.center().y + offset,
            ),
            normal,
        )
    } else {
        let normal = vec2(0.0, if delta.y >= 0.0 { 1.0 } else { -1.0 });
        (
            pos2(
                rect.center().x + offset,
                if normal.y > 0.0 {
                    rect.bottom()
                } else {
                    rect.top()
                },
            ),
            normal,
        )
    }
}
pub(super) fn links_for<'a>(
    p: &'a Project,
    sid: &str,
    scene: &Scene,
    compact: bool,
) -> Vec<Link<'a>> {
    let Some(system) = p.system(sid) else {
        return vec![];
    };
    let groups = if compact {
        groups(system)
    } else {
        system.edges.iter().map(|e| vec![e]).collect()
    };
    let pairs: BTreeSet<_> = system
        .edges
        .iter()
        .filter_map(|e| Some((e.from.node.as_ref()?.clone(), e.to.node.as_ref()?.clone())))
        .collect();
    let mut out = vec![];
    let mut lanes = BTreeMap::<String, usize>::new();
    for group in groups {
        let edge = group[0];
        let lane = lanes
            .entry(edge.from.node.clone().unwrap_or_default())
            .or_default();
        let mut route = scene.route(&edge.from, &edge.to, *lane);
        if edge.from.node.is_some() && edge.from.node == edge.to.node {
            *lane += 1;
        }
        if compact
            && let (Some(a), Some(b)) = (&edge.from.node, &edge.to.node)
            && a != b
        {
            let ar = scene.cards.iter().find(|c| c.id == *a).map(|c| c.rect);
            let br = scene.cards.iter().find(|c| c.id == *b).map(|c| c.rect);
            if let (Some(ar), Some(br)) = (ar, br) {
                let reciprocal = pairs.contains(&(b.clone(), a.clone()));
                let offset = if reciprocal {
                    if a < b { -12.0 } else { 12.0 }
                } else {
                    0.0
                };
                let (ap, an) = summary_anchor(ar, br.center(), offset);
                let (bp, bn) = summary_anchor(br, ar.center(), offset);
                route = Some(Path::between(ap, an, bp, bn));
            }
        }
        let Some(path) = route else {
            continue;
        };
        let label = if compact {
            format!(
                "{} {}",
                group.len(),
                if group.len() == 1 {
                    "connection"
                } else {
                    "connections"
                }
            )
        } else {
            edge.label
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    p.port(sid, &edge.from)
                        .and_then(|port| port.contract.as_ref())
                        .map(ToString::to_string)
                        .unwrap_or_default()
                })
        };
        let selection = if group.len() > 1 {
            Selection::Summary(
                edge.from.node.clone().expect("grouped source"),
                edge.to.node.clone().expect("grouped destination"),
            )
        } else {
            Selection::Edge(edge.id.clone())
        };
        out.push(Link {
            selection,
            edge,
            members: group.iter().map(|e| e.id.clone()).collect(),
            path,
            label,
        });
    }
    out
}

impl Link<'_> {
    pub fn selected(&self, selected: &Selection) -> bool {
        self.selection == *selected
            || matches!(selected, Selection::Edge(id) if self.members.contains(id))
            || matches!(selected, Selection::Summary(a,b) if self.edge.from.node.as_ref()==Some(a) && self.edge.to.node.as_ref()==Some(b))
    }
    pub fn focused_count(&self, emphasis: &focus::Emphasis) -> usize {
        self.members.iter().filter(|id| emphasis.edge(id)).count()
    }
    pub fn caption(&self, emphasis: &focus::Emphasis) -> String {
        let focused = self.focused_count(emphasis);
        if self.members.len() > 1 && emphasis.active && focused > 0 && focused < self.members.len()
        {
            format!(
                "{} · {} of {} in focus",
                self.label,
                focused,
                self.members.len()
            )
        } else {
            self.label.clone()
        }
    }
    pub fn animate(
        &self,
        motion: motion::Motion,
        selected: &Selection,
        emphasis: &focus::Emphasis,
        p: &Project,
        sid: &str,
    ) -> bool {
        p.system(sid).is_some_and(|s| {
            s.edges.iter().any(|e| {
                self.members.contains(&e.id) && emphasis.edge(&e.id) && motion.includes(e, selected)
            })
        })
    }
}
