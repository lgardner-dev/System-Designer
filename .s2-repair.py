from pathlib import Path
p=Path('examples/s2-flow-candidate/view.rs')
s=p.read_text()
def change(old,new,count=1):
    global s
    assert s.count(old)==count, (old,s.count(old))
    s=s.replace(old,new)
change('    contract: Option<String>,\n','')
change('                contract: None,\n','')
change('                contract: Some(a.contract.clone()),\n','')
change('fn control_routes(flow: &Flow) -> Vec<Route> {', '''fn decision_exit(r: Rect, fraction: f32) -> Pos2 {
    let x = r.left() + r.width() * fraction;
    let y = r.center().y + r.height() * 0.5
        * (1.0 - ((x - r.center().x) / (r.width() * 0.5)).abs());
    pos2(x, y)
}
fn diamond(step: &Step, tab: Tab) -> bool {
    step.kind == Kind::Choice && tab == Tab::Control
}
fn control_routes(flow: &Flow) -> Vec<Route> {''')
change('''                    let x = a.left() + a.width() * if e.source_flow == "S2.e9" { 0.3 } else { 0.7 };
                    let start = pos2(x, a.bottom());''', '''                    let start = decision_exit(a, if e.source_flow == "S2.e9" { 0.3 } else { 0.7 });''')
change('''        let role = if step.id == "step.S2.decide" {''', '''        let role = if step.id == flow.entry {
            "Entry - bounded work"
        } else if step.id == "step.S2.decide" {''')
change('''        if step.kind == Kind::Choice {
            painter.add''','''        if diamond(step, app.at.tab) {
            painter.add''')
change('''        let width = if step.kind == Kind::Choice {''','''        let width = if diamond(step, app.at.tab) {''')
change('''        if pointer.is_some_and(|p| r.contains(p) && area.contains(p)) {''', '''        if pointer.is_some_and(|p| {
            let inside = if diamond(step, app.at.tab) {
                ((p.x - r.center().x) / (r.width() * 0.5)).abs()
                    + ((p.y - r.center().y) / (r.height() * 0.5)).abs() <= 1.0
            } else { r.contains(p) };
            inside && area.contains(p)
        }) {''')
s+='''
#[cfg(test)]
mod geometry_tests {
    use super::*;
    use crate::model::Document;

    #[test]
    fn branch_exits_touch_the_drawn_diamond() {
        let r = Rect::from_min_size(pos2(10.0, 20.0), vec2(240.0, 140.0));
        for fraction in [0.3, 0.5, 0.7] {
            let p = decision_exit(r, fraction);
            let normalized = ((p.x - r.center().x) / (r.width() * 0.5)).abs()
                + ((p.y - r.center().y) / (r.height() * 0.5)).abs();
            assert!((normalized - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn interface_cards_do_not_inherit_control_diamonds() {
        let doc = Document::load().unwrap();
        let step = doc.study.flows[0].step("step.S2.enough").unwrap();
        assert!(diamond(step, Tab::Control));
        assert!(!diamond(step, Tab::Interfaces));
    }

    #[test]
    fn every_choice_route_touches_its_choice_in_both_control_views() {
        let doc = Document::load().unwrap();
        for flow in [doc.original(), doc.study.flows[0].clone()] {
            for route in control_routes(&flow) {
                let transition = flow.transition(&route.transition).unwrap();
                for (id, point) in [
                    (&transition.from, route.points.first().unwrap()),
                    (&transition.to, route.points.last().unwrap()),
                ] {
                    let step = flow.step(id).unwrap();
                    if step.kind == Kind::Choice {
                        let r = rect(step);
                        let n = ((point.x - r.center().x) / (r.width() * 0.5)).abs()
                            + ((point.y - r.center().y) / (r.height() * 0.5)).abs();
                        assert!((n - 1.0).abs() < 0.001, "{}: {}", route.id, n);
                    }
                }
            }
        }
    }
}
'''
p.write_text(s,encoding='utf-8',newline='\n')
p=Path('examples/s2-flow-candidate/main.rs')
s=p.read_text().replace('→','->')
s=s.replace('Diamond · recorded choice','Diamond · recorded choice (Control Flow only)')
p.write_text(s,encoding='utf-8',newline='\n')
