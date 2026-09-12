//! SCC-aware layered drawing. Cycle groups exist ONLY in the layout algorithm.
//! No edge is reversed, removed, or interpreted as a prerequisite/execution order.
use super::*;
use std::collections::{BTreeSet, VecDeque};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Axis {
    #[default]
    Horizontal,
    Vertical,
}
#[derive(Clone)]
struct Block {
    members: Vec<usize>,
    positions: BTreeMap<usize, Vec2>,
    size: Vec2,
}
fn graph(p: &Project, sid: &str) -> (Vec<String>, Vec<Vec<usize>>) {
    let Some(system) = p.system(sid) else {
        return (vec![], vec![]);
    };
    let mut ids: Vec<_> = system.nodes.iter().map(|n| n.id.clone()).collect();
    ids.sort();
    let index: BTreeMap<_, _> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let mut adjacency = vec![Vec::new(); ids.len()];
    for edge in &system.edges {
        if let (Some(a), Some(b)) = (&edge.from.node, &edge.to.node)
            && let (Some(&a), Some(&b)) = (index.get(a.as_str()), index.get(b.as_str()))
        {
            adjacency[a].push(b);
        }
    }
    (
        ids,
        adjacency
            .into_iter()
            .map(|mut edges| {
                edges.sort_unstable();
                edges
            })
            .collect(),
    )
}
/// Iterative Kosaraju: component-tree depth and graph depth do not use the call stack.
fn components(adjacency: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adjacency.len();
    let mut seen = vec![false; n];
    let mut finish = vec![];
    for root in 0..n {
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut stack = vec![(root, 0)];
        while let Some((v, index)) = stack.last_mut() {
            if *index < adjacency[*v].len() {
                let next = adjacency[*v][*index];
                *index += 1;
                if !seen[next] {
                    seen[next] = true;
                    stack.push((next, 0));
                }
            } else {
                let (v, _) = stack.pop().expect("nonempty DFS stack");
                finish.push(v);
            }
        }
    }
    let mut reverse = vec![vec![]; n];
    for (a, targets) in adjacency.iter().enumerate() {
        for &b in targets {
            reverse[b].push(a);
        }
    }
    seen.fill(false);
    let mut result = vec![];
    for &root in finish.iter().rev() {
        if seen[root] {
            continue;
        }
        seen[root] = true;
        let mut pending = vec![root];
        let mut group = vec![];
        while let Some(v) = pending.pop() {
            group.push(v);
            for &next in &reverse[v] {
                if !seen[next] {
                    seen[next] = true;
                    pending.push(next);
                }
            }
        }
        group.sort();
        result.push(group);
    }
    result.sort_by_key(|g| g[0]);
    result
}
/// A ring prevents a genuine feedback network being forced into a false linear story.
fn block(members: Vec<usize>, sizes: &[Vec2], axis: Axis, adjacency: &[Vec<usize>]) -> Block {
    let mut order = members.clone();
    // Greedy neighbour ordering shortens likely ring chords; stable identity breaks ties.
    if order.len() > 3 {
        let mut remaining: BTreeSet<_> = order.iter().copied().collect();
        let mut sorted = vec![];
        let mut current = order[0];
        remaining.remove(&current);
        sorted.push(current);
        while !remaining.is_empty() {
            let next = remaining
                .iter()
                .copied()
                .max_by_key(|&v| {
                    (
                        (adjacency[current].contains(&v) as u8)
                            + (adjacency[v].contains(&current) as u8),
                        std::cmp::Reverse(v),
                    )
                })
                .expect("remaining");
            remaining.remove(&next);
            sorted.push(next);
            current = next;
        }
        order = sorted;
    }
    let mut positions = BTreeMap::new();
    if order.len() == 1 {
        positions.insert(order[0], Vec2::ZERO);
    } else if order.len() == 2 {
        positions.insert(order[0], Vec2::ZERO);
        let size = sizes[order[0]];
        positions.insert(
            order[1],
            if axis == Axis::Horizontal {
                vec2(0.0, size.y + 120.0)
            } else {
                vec2(size.x + 140.0, 0.0)
            },
        );
    } else {
        let diameter = order
            .iter()
            .map(|&i| sizes[i].length())
            .fold(0.0_f32, f32::max)
            + 120.0;
        let radius = diameter / (2.0 * (std::f32::consts::PI / order.len() as f32).sin());
        for (j, &i) in order.iter().enumerate() {
            let angle = 2.0 * std::f32::consts::PI * j as f32 / order.len() as f32
                - std::f32::consts::FRAC_PI_2;
            positions.insert(i, vec2(angle.cos(), angle.sin()) * radius - sizes[i] * 0.5);
        }
    }
    let bounds = positions.iter().fold(Rect::NOTHING, |r, (&i, pos)| {
        r.union(Rect::from_min_size(pos2(pos.x, pos.y), sizes[i]))
    });
    for pos in positions.values_mut() {
        *pos -= bounds.min.to_vec2();
    }
    Block {
        members,
        positions,
        size: bounds.size(),
    }
}
fn place(
    ids: &[String],
    adjacency: &[Vec<usize>],
    sizes: &[Vec2],
    axis: Axis,
    child: bool,
) -> BTreeMap<String, Position> {
    if ids.is_empty() {
        return BTreeMap::new();
    }
    let blocks: Vec<_> = components(adjacency)
        .into_iter()
        .map(|g| block(g, sizes, axis, adjacency))
        .collect();
    let mut owner = vec![0; ids.len()];
    for (i, b) in blocks.iter().enumerate() {
        for &v in &b.members {
            owner[v] = i;
        }
    }
    let mut weights = BTreeMap::<(usize, usize), usize>::new();
    let mut next = vec![BTreeSet::new(); blocks.len()];
    let mut prev = next.clone();
    for (a, targets) in adjacency.iter().enumerate() {
        for &b in targets {
            let (u, v) = (owner[a], owner[b]);
            if u != v {
                next[u].insert(v);
                prev[v].insert(u);
                *weights.entry((u, v)).or_default() += 1;
            }
        }
    }
    let mut indegree: Vec<_> = prev.iter().map(BTreeSet::len).collect();
    let mut pending: VecDeque<_> = (0..blocks.len()).filter(|&i| indegree[i] == 0).collect();
    let mut ranks = vec![0_usize; blocks.len()];
    while let Some(v) = pending.pop_front() {
        for &other in &next[v] {
            ranks[other] = ranks[other].max(ranks[v] + 1);
            indegree[other] -= 1;
            if indegree[other] == 0 {
                pending.push_back(other);
            }
        }
    }
    let mut layers = vec![vec![]; ranks.iter().max().copied().unwrap_or(0) + 1];
    for (i, &rank) in ranks.iter().enumerate() {
        layers[rank].push(i);
    }
    // A few barycenter sweeps reduce crossings without an unbounded optimizer.
    for sweep in 0..8 {
        let order: BTreeMap<_, _> = layers
            .iter()
            .flat_map(|layer| layer.iter().enumerate().map(|(j, &b)| (b, j as f32)))
            .collect();
        for layer in &mut layers {
            layer.sort_by(|&a, &b| {
                let score = |i: usize| {
                    let neighbours = if sweep % 2 == 0 { &prev[i] } else { &next[i] };
                    if neighbours.is_empty() {
                        order[&i]
                    } else {
                        let weight = |n: &usize| {
                            *weights
                                .get(&(if sweep % 2 == 0 { (*n, i) } else { (i, *n) }))
                                .unwrap_or(&1) as f32
                        };
                        neighbours.iter().map(|n| order[n] * weight(n)).sum::<f32>()
                            / neighbours.iter().map(weight).sum::<f32>()
                    }
                };
                score(a).total_cmp(&score(b)).then(a.cmp(&b))
            });
        }
    }
    let along = |s: Vec2| if axis == Axis::Horizontal { s.x } else { s.y };
    let across = |s: Vec2| if axis == Axis::Horizontal { s.y } else { s.x };
    let heights: Vec<f32> = layers
        .iter()
        .map(|layer| {
            layer.iter().map(|&i| across(blocks[i].size)).sum::<f32>()
                + 120.0 * layer.len().saturating_sub(1) as f32
        })
        .collect();
    let maximum = heights.iter().copied().fold(0.0_f32, f32::max);
    let mut forward = 0.0_f32;
    let mut positions = BTreeMap::new();
    for (rank, layer) in layers.iter().enumerate() {
        let mut cross = (maximum - heights[rank]) * 0.5;
        for &i in layer {
            let origin = if axis == Axis::Horizontal {
                vec2(forward, cross)
            } else {
                vec2(cross, forward)
            };
            for (&v, offset) in &blocks[i].positions {
                let p = origin + *offset + vec2(if child { 280.0 } else { 100.0 }, 150.0);
                positions.insert(
                    ids[v].clone(),
                    Position {
                        x: p.x as f64,
                        y: p.y as f64,
                    },
                );
            }
            cross += across(blocks[i].size) + 120.0;
        }
        forward += layer
            .iter()
            .map(|&i| along(blocks[i].size))
            .fold(0.0_f32, f32::max)
            + 180.0;
    }
    positions
}
pub(super) fn arranged(p: &Project, sid: &str, axis: Axis) -> Result<BTreeMap<String, Position>> {
    let (ids, adjacency) = graph(p, sid);
    if ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let initial = Scene::new(p, sid, &auto_layout(p, sid));
    let mut sizes: Vec<_> = ids
        .iter()
        .map(|id| {
            initial
                .cards
                .iter()
                .find(|c| c.id == *id)
                .map(|c| c.rect.size())
                .unwrap_or(vec2(300.0, 160.0))
        })
        .collect();
    // Changing neighbours may change which side owns the label slots. Grow the
    // measured envelope until the actual projected cards fit their reserved cells.
    for _ in 0..16 {
        let positions = place(&ids, &adjacency, &sizes, axis, sid != p.root);
        let projected = Scene::new(p, sid, &positions);
        let mut grew = false;
        for (i, id) in ids.iter().enumerate() {
            if let Some(card) = projected.cards.iter().find(|c| c.id == *id) {
                let actual = card.rect.size();
                if actual.x > sizes[i].x + 0.01 || actual.y > sizes[i].y + 0.01 {
                    sizes[i] = sizes[i].max(actual);
                    grew = true;
                }
            }
        }
        if !grew {
            return Ok(positions);
        }
    }
    Err(ModelError::one(
        "Connection layout did not settle its card sizes. Existing positions were preserved.",
    ))
}
pub(super) fn apply(app: &mut Designer, axis: Option<Axis>) {
    app.canvas.cancel();
    let sid = app.current.clone();
    let positions = match axis {
        Some(axis) => arranged(app.store.project(), &sid, axis),
        None => grid(app.store.project(), &sid),
    };
    let candidate = positions.and_then(|positions| {
        edit::candidate(app.store.project(), |p| {
            p.layout.insert(sid, positions);
            Ok(())
        })
    });
    let succeeded = candidate.is_ok();
    app.publish(
        if axis.is_some() {
            "Arrange by connections"
        } else {
            "Arrange in grid"
        },
        candidate,
    );
    if succeeded && app.error.is_none() {
        app.canvas.fit_requested = true;
    }
}
/// Grid also measures four-sided Detail footprints, not the obsolete left/right size.
fn grid(p: &Project, sid: &str) -> Result<BTreeMap<String, Position>> {
    let system = p
        .system(sid)
        .ok_or_else(|| ModelError::one("Missing layout system"))?;
    let mut positions = auto_layout(p, sid);
    let columns = ((system.nodes.len() as f32 / 1.25).sqrt().ceil() as usize).max(1);
    let mut width = 300.0_f32;
    let mut height = 160.0_f32;
    for _ in 0..16 {
        let scene = Scene::new(p, sid, &positions);
        let new_width = scene
            .cards
            .iter()
            .map(|c| c.rect.width())
            .fold(width, f32::max);
        let new_height = scene
            .cards
            .iter()
            .map(|c| c.rect.height())
            .fold(height, f32::max);
        width = new_width;
        height = new_height;
        positions = system
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    n.id.clone(),
                    Position {
                        x: (if sid == p.root { 100.0 } else { 280.0 })
                            + (i % columns) as f64 * (width + 160.0) as f64,
                        y: 150.0 + (i / columns) as f64 * (height + 140.0) as f64,
                    },
                )
            })
            .collect();
        let measured = Scene::new(p, sid, &positions);
        if measured
            .cards
            .iter()
            .all(|c| c.rect.width() <= width + 0.01 && c.rect.height() <= height + 0.01)
        {
            return Ok(positions);
        }
    }
    Err(ModelError::one(
        "Grid sizes did not settle; previous positions were preserved.",
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Project {
        parse(include_str!("../../../tests/fixtures/project.json")).expect("fixture")
    }
    #[test]
    fn cycles_group_without_reversing_edges() {
        assert_eq!(
            components(&[vec![1], vec![0, 2], vec![]]),
            vec![vec![0, 1], vec![2]]
        );
    }
    #[test]
    fn deep_graph_uses_no_recursive_dfs() {
        let mut g = vec![vec![]; 4096];
        for (i, successors) in g.iter_mut().enumerate().take(4095) {
            successors.push(i + 1);
        }
        assert_eq!(components(&g).len(), 4096);
    }
    #[test]
    fn condensation_ranks_follow_forward_edges() {
        let ids = vec!["a".into(), "b".into(), "c".into()];
        let g = vec![vec![1], vec![2], vec![]];
        let positions = place(&ids, &g, &[vec2(300.0, 150.0); 3], Axis::Horizontal, false);
        assert!(positions["a"].x < positions["b"].x && positions["b"].x < positions["c"].x);
    }
    #[test]
    fn vertical_orientation_uses_y_ranks() {
        let ids = vec!["a".into(), "b".into()];
        let positions = place(
            &ids,
            &[vec![1], vec![]],
            &[vec2(300.0, 150.0); 2],
            Axis::Vertical,
            false,
        );
        assert!(positions["a"].y < positions["b"].y);
    }
    #[test]
    fn layout_is_deterministic() {
        let p = fixture();
        assert_eq!(
            arranged(&p, "root", Axis::Horizontal).expect("layout"),
            arranged(&p, "root", Axis::Horizontal).expect("layout")
        );
    }
    #[test]
    fn leaf_and_empty_system_work() {
        let p = Project::blank();
        assert!(
            arranged(&p, "root", Axis::Horizontal)
                .expect("empty")
                .is_empty()
        );
        let p = fixture();
        assert!(arranged(&p, "a", Axis::Horizontal).is_ok());
    }
    #[test]
    fn application_design_has_no_overlapping_cards() {
        let p = parse(crate::APPLICATION_DESIGN).expect("design");
        for s in &p.systems {
            for axis in [Axis::Horizontal, Axis::Vertical] {
                let positions = arranged(&p, &s.id, axis).expect("layout");
                let scene = Scene::new(&p, &s.id, &positions);
                for (i, a) in scene.cards.iter().enumerate() {
                    for b in &scene.cards[i + 1..] {
                        assert!(
                            !a.rect.expand(5.0).intersects(b.rect.expand(5.0)),
                            "{} {} overlap",
                            a.id,
                            b.id
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn all_edges_contracts_and_scope_base_are_preserved() {
        use crate::exchange::{Scope, export};
        let p = fixture();
        let packet = serde_json::to_value(export(&p, "root", Scope::Level, None).expect("export"))
            .expect("json");
        let mut q = p.clone();
        q.layout.insert(
            "root".into(),
            arranged(&p, "root", Axis::Horizontal).expect("layout"),
        );
        assert_eq!(q.systems, p.systems);
        assert_eq!(q.contracts, p.contracts);
        assert_eq!(
            packet,
            serde_json::to_value(export(&q, "root", Scope::Level, None).expect("export"))
                .expect("json")
        );
    }
    #[test]
    fn arrangement_is_one_undoable_local_edit() {
        let p = fixture();
        let mut store = edit::Store::new(p.clone()).expect("store");
        let mut q = p.clone();
        q.layout.insert(
            "a".into(),
            arranged(&p, "a", Axis::Horizontal).expect("layout"),
        );
        store.publish("layout", q).expect("publish");
        store.undo();
        assert_eq!(store.project(), &p);
        store.redo();
        assert_ne!(store.project(), &p);
    }
}
