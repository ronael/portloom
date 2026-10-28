use crate::domain::{
    assessment::{Kind, assess},
    policy::blocked,
};
use crate::presentation::{design::*, state::State};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::Modifier,
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
pub fn kind_color(kind: Kind) -> ratatui::style::Color {
    match kind {
        Kind::Web | Kind::Development => ACCENT,
        Kind::System => SUBTLE,
        Kind::Application => MUTED,
        _ => WARNING,
    }
}
pub fn render(frame: &mut Frame, state: &mut State, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style(BORDER))
        .style(style(TEXT).bg(SURFACE))
        .title(
            Line::from(format!(
                " PORTS EN ÉCOUTE  {} / {} ",
                state.visible.len(),
                state.snapshot.endpoints.len()
            ))
            .style(style(MUTED)),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);
    state.table_rect = Rect::new(
        inner.x,
        inner.y + 2,
        inner.width,
        inner.height.saturating_sub(2),
    );
    state.page_size = state.table_rect.height as usize;
    let title_width = inner.width.saturating_sub(31) as usize;
    let rows = state
        .visible
        .iter()
        .map(|index| {
            let e = &state.snapshot.endpoints[*index];
            let a = assess(&e.process);
            let action = if blocked(&e.process, &state.actor).is_some() {
                "PROTÉGÉ"
            } else if a.needs_pid {
                "PRUDENCE"
            } else {
                "CONFIRMER"
            };
            Row::new(vec![
                Cell::from(format!("{}", e.port)).style(style(TEXT).add_modifier(Modifier::BOLD)),
                Cell::from(e.protocol.label()).style(style(SUBTLE)),
                Cell::from(a.badge).style(style(kind_color(a.kind))),
                Cell::from(fit(&a.title, title_width)),
                Cell::from(action).style(style(if action == "PROTÉGÉ" {
                    SUBTLE
                } else if a.needs_pid {
                    WARNING
                } else {
                    MUTED
                })),
            ])
        })
        .collect::<Vec<_>>();
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(8),
            Constraint::Length(9),
        ],
    )
    .column_spacing(1)
    .header(
        Row::new(["PORT", "NET", "USAGE", "PROJET / SERVICE", "ARRÊT"])
            .style(style(SUBTLE))
            .bottom_margin(1),
    )
    .row_highlight_style(style(TEXT).bg(SELECTED));
    frame.render_stateful_widget(table, inner, &mut state.table);
    if state.visible.is_empty() {
        frame.render_widget(
            Paragraph::new(if state.busy {
                "Lecture des ports…"
            } else {
                "Aucun port dans cette vue.\n0 : tout afficher · Échap : effacer le filtre"
            })
            .style(style(MUTED)),
            state.table_rect,
        );
    }
}
