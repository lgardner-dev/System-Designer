use crate::model::*;
use std::sync::Arc;
#[derive(Clone)]
struct Snapshot {
    label: String,
    project: Arc<Project>,
}
/// The single publication boundary. Invalid candidates never mutate current or history.
pub struct Store {
    project: Arc<Project>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    pub generation: u64,
}
impl Store {
    pub fn new(project: Project) -> Result<Self> {
        validate(&project)?;
        Ok(Self {
            project: Arc::new(project),
            undo: vec![],
            redo: vec![],
            generation: 0,
        })
    }
    pub fn project(&self) -> &Project {
        &self.project
    }
    /// Cheap immutable snapshot for a UI frame; it cannot mutate Store.
    pub fn snapshot(&self) -> Arc<Project> {
        Arc::clone(&self.project)
    }
    pub fn publish(&mut self, label: impl Into<String>, candidate: Project) -> Result<()> {
        validate(&candidate)?;
        if candidate == *self.project {
            return Ok(());
        }
        self.undo.push(Snapshot {
            label: label.into(),
            project: self.project.clone(),
        });
        if self.undo.len() > 50 {
            self.undo.remove(0);
        }
        self.project = Arc::new(candidate);
        self.redo.clear();
        self.generation = self.generation.wrapping_add(1);
        Ok(())
    }
    pub fn undo_label(&self) -> Option<&str> {
        self.undo.last().map(|s| s.label.as_str())
    }
    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|s| s.label.as_str())
    }
    pub fn undo(&mut self) -> bool {
        let Some(s) = self.undo.pop() else {
            return false;
        };
        self.redo.push(Snapshot {
            label: s.label,
            project: std::mem::replace(&mut self.project, s.project),
        });
        self.generation = self.generation.wrapping_add(1);
        true
    }
    pub fn redo(&mut self) -> bool {
        let Some(s) = self.redo.pop() else {
            return false;
        };
        self.undo.push(Snapshot {
            label: s.label,
            project: std::mem::replace(&mut self.project, s.project),
        });
        self.generation = self.generation.wrapping_add(1);
        true
    }
}
