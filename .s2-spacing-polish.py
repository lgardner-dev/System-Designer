from pathlib import Path

p = Path('examples/s2-flow-candidate/view.rs')
s = p.read_text()
old = '''    let size = match s.kind {
        Kind::Choice => vec2(240.0, 140.0),
        Kind::Entry | Kind::Outcome => vec2(210.0, 84.0),
        _ => vec2(240.0, 108.0),
    };'''
new = '''    let size = match s.kind {
        Kind::Choice => vec2(240.0, 140.0),
        // Boundary entry markers stay compact so authored process positions do not overlap.
        Kind::Entry => vec2(110.0, 80.0),
        Kind::Outcome => vec2(170.0, 80.0),
        _ => vec2(240.0, 108.0),
    };'''
assert old in s
s = s.replace(old, new)
old = '''            let point = if route.transition == "S2.e10" {
                transform(pos2(190.0, 350.0))
            } else {
                at(&points, len * 0.5).0 - vec2(0.0, 14.0)
            };'''
new = '''            let point = at(&points, len * 0.5).0 - vec2(0.0, 14.0);'''
assert old in s
s = s.replace(old, new)
p.write_text(s, encoding='utf-8', newline='\n')
