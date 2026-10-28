use crate::presentation::{
    design::{ACCENT, DANGER, MUTED, WARNING, style},
    state::{Overlay, State},
};
use ratatui::{Frame, layout::Rect, style::Modifier, text::Line, widgets::Paragraph};

/// Pip: an original seven-cell network plug, drawn within the existing header.
/// Expressions follow application state; no independent animation loop is needed.
pub fn render(frame: &mut Frame, state: &State, area: Rect) {
    let (face, color) = if let Overlay::Confirm { plan, .. } = &state.overlay {
        ("│ ! ! │", if plan.force { DANGER } else { WARNING })
    } else if state.stale || state.error {
        ("│ ! ! │", DANGER)
    } else if state.busy {
        ("│ ◔ ◔ │", ACCENT)
    } else if state.paused {
        ("│ -_- │", MUTED)
    } else if state.searching {
        ("│ › ‹ │", ACCENT)
    } else {
        ("│ •_• │", ACCENT)
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from("╭─────╮"),
            Line::from(face),
            Line::from("╰─┬─┬─╯"),
        ])
        .style(style(color).add_modifier(Modifier::BOLD)),
        area,
    );
}
