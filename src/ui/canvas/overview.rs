//! Reversible visual summaries. Each summary retains every member edge ID.
use super::*;
use crate::model::Edge;
use std::collections::BTreeSet;
const ID: &str = "system-designer.compact-overview";
#[derive(Clone, Default)]
pub(super) struct State {
    pub compact: bool,
    project: String,
    system: String,
    pub group: Vec<String>,
}
fn load(ctx: &egui::Context) -> State {
    ctx.data(|d| d.get_temp::<State>(egui::Id::new(ID)))
        .unwrap_or_default()
}
fn save(ctx: &egui::Context, state: State) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID), state));
}
pub(super) fn controls(app: &mut Designer, ui: &mut egui::Ui) -> State {
    let mut state = load(ui.ctx());
    let p = app.store.project();
    if state.project != p.id || state.system != app.current {
        state.project = p.id.clone();
        state.system = app.current.clone();
        state.group.clear();
    }
    if matches!(app.selected, Selection::Node(_) | Selection::Boundary(_)) {
        state.group.clear();
    }
    let before = state.compact;
    ui.horizontal_wrapped(|ui| {
        ui.strong("B · Compact overview");
        ui.selectable_value(&mut state.compact, true, "Overview");
        ui.selectable_value(&mut state.compact, false, "Detail / wire ports");
        ui.small("One arrow summarizes one direction; a count is not a shared contract.");
    });
    if before != state.compact {
        app.canvas.cancel();
    }
    if state.compact {
        let n = p.system(&app.current).map_or(0, |s| s.edges.len());
        ui.small(format!("All {n} connections retained. Select a summary to inspect every member; use Detail to wire exact ports. Exports remain complete."));
    } else {
        ui.small("Exact ports and individual connections. Overview is a drawing projection only; it never rewrites the design.");
    }
    save(ui.ctx(), state.clone());
    state
}
#[derive(Clone)]
pub(super) struct Link<'a> {
    pub edge: &'a Edge,
    pub members: Vec<String>,
    pub path: Path,
    pub label: String,
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
        *lane += 1;
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
        out.push(Link {
            edge,
            members: group.iter().map(|e| e.id.clone()).collect(),
            path,
            label,
        });
    }
    out
}
pub(super) fn select(app: &mut Designer, ctx: &egui::Context, members: &[String]) {
    let mut state = load(ctx);
    state.project = app.store.project().id.clone();
    state.system = app.current.clone();
    state.group = members.to_vec();
    app.selected = if members.len() == 1 {
        Selection::Edge(members[0].clone())
    } else {
        Selection::None
    };
    save(ctx, state);
}
pub(super) fn expand(ctx: &egui::Context) {
    let mut state = load(ctx);
    state.compact = false;
    save(ctx, state);
}
pub(super) fn clear(ctx: &egui::Context) {
    let mut state = load(ctx);
    state.group.clear();
    save(ctx, state);
}
fn full_endpoint(p: &Project, sid: &str, ep: &Endpoint) -> String {
    let name = ep
        .node
        .as_ref()
        .and_then(|id| p.node(id))
        .map(|(_, n)| n.name.as_str())
        .unwrap_or("Boundary");
    let port = p
        .port(sid, ep)
        .map(|p| p.name.as_str())
        .unwrap_or("Missing");
    format!("{name} / {port}")
}
pub(in crate::ui) fn details(app: &mut Designer, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let mut state = load(&ctx);
    let p = app.store.snapshot();
    if matches!(app.selected, Selection::Node(_) | Selection::Boundary(_)) {
        state.group.clear();
        save(&ctx, state.clone());
    }
    if !state.compact
        || state.project != p.id
        || state.system != app.current
        || state.group.is_empty()
    {
        return;
    }
    let members: Vec<_> = p
        .system(&app.current)
        .into_iter()
        .flat_map(|s| &s.edges)
        .filter(|e| state.group.contains(&e.id))
        .collect();
    ui.heading(format!("{} exact connections", members.len()));
    ui.small("Visual summary only. Select a specific member before editing or deleting.");
    if ui.button("Expand into Detail").clicked() {
        state.compact = false;
        app.canvas.cancel();
    }
    egui::ScrollArea::vertical()
        .id_salt("overview-members")
        .max_height(300.0)
        .show(ui, |ui| {
            for edge in members {
                ui.push_id(&edge.id, |ui| {
                    ui.strong(&edge.id);
                    ui.label(format!(
                        "{} -> {}",
                        full_endpoint(&p, &app.current, &edge.from),
                        full_endpoint(&p, &app.current, &edge.to)
                    ));
                    if let Some(r) = p
                        .port(&app.current, &edge.from)
                        .and_then(|r| r.contract.as_ref())
                    {
                        ui.label(r.to_string());
                        if let Some(c) = p.contract(r) {
                            ui.small(&c.name);
                        }
                    }
                    if let Some(label) = &edge.label {
                        ui.label(label);
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Show exact wire").clicked() {
                            state.compact = false;
                            app.selected = Selection::Edge(edge.id.clone());
                            app.canvas.cancel();
                        }
                        if ui.button("Edit this wire").clicked() {
                            app.selected = Selection::Edge(edge.id.clone());
                            app.dialog = Some(Dialog::Connection(ConnectionDialog::new(
                                &p,
                                &app.current,
                                None,
                                Some(&edge.id),
                            )));
                        }
                    });
                    ui.separator();
                });
            }
        });
    save(&ctx, state);
    ui.separator();
}
#[cfg(test)]
pub(super) fn enable_for_test(ctx: &egui::Context) {
    let mut s = load(ctx);
    s.compact = true;
    save(ctx, s);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project() -> Project {
        parse(include_str!("../../../tests/fixtures/project.json")).expect("fixture")
    }
    #[test]
    fn members_partition_edges_exactly() {
        let mut p = project();
        let mut e = p.system("root").expect("root").edges[0].clone();
        e.id = "parallel".into();
        p.system_mut("root").expect("root").edges.push(e);
        let s = p.system("root").expect("root");
        let mut ids: Vec<_> = groups(s).iter().flatten().map(|e| e.id.clone()).collect();
        ids.sort();
        let mut expected: Vec<_> = s.edges.iter().map(|e| e.id.clone()).collect();
        expected.sort();
        assert_eq!(ids, expected);
        assert!(groups(s).iter().any(|g| g.len() == 2));
    }
    #[test]
    fn reciprocal_wires_never_merge() {
        let p = project();
        let s = p.system("root").expect("root");
        assert!(groups(s).iter().all(|g| {
            g.iter()
                .all(|e| e.from.node == g[0].from.node && e.to.node == g[0].to.node)
        }));
    }
    #[test]
    fn boundary_ports_never_aggregate() {
        let p = project();
        let s = p.system("a").expect("child");
        for g in groups(s) {
            if g.iter()
                .any(|e| e.from.node.is_none() || e.to.node.is_none())
            {
                assert_eq!(g.len(), 1);
            }
        }
    }
    #[test]
    fn compact_geometry_preserves_owner_ports() {
        let p = project();
        let scene = compact_scene(Scene::new(&p, "a", &auto_layout(&p, "a")));
        assert_eq!(
            scene.ports.iter().filter(|a| a.boundary).count(),
            p.boundary("a").len()
        );
        for a in scene.ports {
            assert!(p.port("a", &a.endpoint).is_some());
        }
    }
    #[test]
    fn summaries_cannot_delete_or_retype_a_group() {
        let ctx = egui::Context::default();
        let mut app = Designer::blank();
        select(&mut app, &ctx, &["e1".into(), "e2".into()]);
        assert_eq!(app.selected, Selection::None);
    }
    #[test]
    fn overview_does_not_filter_ai_exports() {
        use crate::exchange::{Scope, export};
        let p = project();
        let before = serde_json::to_value(export(&p, "root", Scope::Level, None).expect("packet"))
            .expect("json");
        let scene = compact_scene(Scene::new(&p, "root", &auto_layout(&p, "root")));
        let _ = links_for(&p, "root", &scene, true);
        assert_eq!(
            before,
            serde_json::to_value(export(&p, "root", Scope::Level, None).expect("packet"))
                .expect("json")
        );
    }
    #[test]
    fn compact_cards_keep_origins_and_have_small_bounds() {
        let p = project();
        let positions = auto_layout(&p, "root");
        let scene = compact_scene(Scene::new(&p, "root", &positions));
        for c in scene.cards {
            assert_eq!(c.rect.size(), vec2(250.0, 112.0));
            assert_eq!(c.position, positions[&c.id]);
        }
    }
}
