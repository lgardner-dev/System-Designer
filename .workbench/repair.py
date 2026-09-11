from pathlib import Path

path = Path('src/ui/workspace.rs')
source = path.read_text()
assert 'use crate::edit;' not in source
path.write_text('use crate::edit;\n' + source)

path = Path('src/ui/canvas/overview.rs')
source = path.read_text()
old = 'self.selection == *selected || matches!(selected, Selection::Edge(id) if self.members.contains(id))'
assert source.count(old) == 1
source = source.replace(old, old + '\n            || matches!(selected, Selection::Summary(a,b) if self.edge.from.node.as_ref()==Some(a) && self.edge.to.node.as_ref()==Some(b))')
# Only self-loop routes consume perimeter lanes. Ordinary output count must not
# make an unrelated self-loop enormous in either representation.
old = '*lane += 1;'
assert source.count(old) == 1
source = source.replace(old, 'if edge.from.node.is_some() && edge.from.node == edge.to.node { *lane += 1; }')
path.write_text(source)

path = Path('src/ui/canvas.rs')
source = path.read_text()
start = source.index('        ui.horizontal_wrapped(|ui| {\n            ui.strong(')
end = source.index('        // Fixed status row', start)
source = source[:start] + '''        let name = p.owner(&self.current).map(|(_, n)| n.name.as_str()).unwrap_or(&p.name);
        let mut title = format!("{name} · {} components · {} connections", counts.0, counts.1);
        if self.canvas_session.view == View::Overview {
            let strokes = p.system(&self.current).map(|s| overview::groups(s).len()).unwrap_or(0);
            title.push_str(&format!(" · {strokes} summary strokes; all edges retained"));
        }
        if counts.0 > 8 { title.push_str(" · Consider meaningful decomposition"); }
        ui.add(egui::Label::new(egui::RichText::new(&title).strong()).truncate()).on_hover_text(title);
''' + source[end:]
path.write_text(source)
print('Applied compile fix and summary/view stability repairs.')
