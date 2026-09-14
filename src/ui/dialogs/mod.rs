//! Fixed modal header collects intents; forms consume them after processing input.
mod position;
mod show;
use super::{Dialog, flow::FlowDialog};
use eframe::egui::{self, Align, Layout};
pub(super) use position::PositionForm;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Primary,
    Preview,
    Discard,
}
#[derive(Default)]
pub(super) struct Actions {
    intent: Option<Action>,
    pub cancel: bool,
    pub blocked: bool,
}
impl Actions {
    pub fn take(&mut self, action: Action, ready: bool) -> bool {
        if self.intent == Some(action) {
            self.intent = None;
            self.blocked = !ready;
            ready
        } else {
            false
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct Header<'a> {
    pub title: &'a str,
    pub primary: Option<&'a str>,
    pub preview: Option<&'a str>,
}
impl Dialog {
    pub(super) fn header(&self) -> Header<'_> {
        let (title, primary, preview) = match self {
            Self::Position(_) => ("Move selection", Some("Apply position"), None),
            Self::Project { .. } => ("Project purpose", Some("Save details"), None),
            Self::Node(_) => ("Edit component", Some("Save component"), None),
            Self::Connection(d) => (
                "Connect ports",
                Some(if d.id.is_some() {
                    "Apply connection change"
                } else {
                    "Create connection"
                }),
                None,
            ),
            Self::Contract(_) => ("Contract definition", Some("Save contract"), None),
            Self::Catalog => ("Contract catalog", None, None),
            Self::Confirm { .. } => ("Confirm change", Some("Apply change"), None),
            Self::Unsaved(_) => (
                "Keep your changes?",
                Some("Save and continue"),
                Some("Discard and continue"),
            ),
            Self::Recovery => ("Recovery copies", None, None),
            Self::Help => ("Help", None, None),
            Self::Handoff(d) => return d.header(),
            Self::Flow(d) => match d {
                FlowDialog::Start => ("Start a flow", Some("Start flow"), None),
                FlowDialog::Step(_) => ("Edit flow step", Some("Save step"), None),
                FlowDialog::Transition(_) => ("Control transition", Some("Save transition"), None),
                FlowDialog::Primitive(_) => ("Stopping criteria", Some("Save explanation"), None),
                FlowDialog::Extract(_) => (
                    "Extract a responsibility",
                    Some("Create component"),
                    Some("Preview boundary"),
                ),
                FlowDialog::Information(_) => (
                    "Information requirement",
                    Some("Apply information requirement"),
                    Some("Review complete edit"),
                ),
                FlowDialog::Port(_) => (
                    "Refine port contract",
                    Some("Apply contract refinement"),
                    Some("Review complete edit"),
                ),
                FlowDialog::Delete(_) => {
                    ("Confirm flow removal", Some("Delete listed items"), None)
                }
                FlowDialog::Uses(_) => ("Uses in Control Flow", None, None),
            },
        };
        Header {
            title,
            primary,
            preview,
        }
    }
}
pub(super) fn header(
    ui: &mut egui::Ui,
    spec: Header<'_>,
    status: &str,
    error: Option<&str>,
) -> Actions {
    let mut actions = Actions::default();
    // The action row wraps within logical viewport space; it is never in the body ScrollArea.
    ui.horizontal(|ui| {
        ui.heading(spec.title);
    });
    ui.horizontal_wrapped(|ui| {
        ui.with_layout(
            Layout::right_to_left(Align::Center).with_main_wrap(true),
            |ui| {
                if let Some(label) = spec.primary
                    && ui.button(label).clicked()
                {
                    actions.intent = Some(Action::Primary);
                }
                if let Some(label) = spec.preview
                    && ui.button(label).clicked()
                {
                    actions.intent = Some(if label == "Discard and continue" {
                        Action::Discard
                    } else {
                        Action::Preview
                    });
                }
                actions.cancel = ui
                    .button(if spec.primary.is_some() {
                        "Cancel"
                    } else {
                        "Close"
                    })
                    .clicked();
            },
        );
    });
    ui.small("Ctrl/Cmd+Enter: primary action · Escape: cancel · Save details changes the document; Ctrl/Cmd+S outside this form saves the file.");
    if let Some(error) = error {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
    } else {
        ui.small(status);
    }
    ui.separator();
    if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter))
        && spec.primary.is_some()
    {
        actions.intent = Some(Action::Primary);
    }
    actions
}
