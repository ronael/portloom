use crate::domain::{
    assessment::{Kind, assess},
    policy::blocked,
};
use crate::presentation::{design::*, grouping::VisibleRow, i18n::T, state::State};
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
                " {}  {} / {} ",
                state.locale.text(if state.grouped {
                    T::Projects
                } else {
                    T::Listeners
                }),
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
        .rows
        .iter()
        .map(|row| {
            let index = match row {
                VisibleRow::Group(group) => {
                    let expanded =
                        !state.query.is_empty() || !state.collapsed.contains(&group.root);
                    let branch = group
                        .endpoints
                        .first()
                        .and_then(|i| state.snapshot.endpoints[*i].process.project.as_ref())
                        .and_then(|p| p.branch.as_deref());
                    let name = match branch {
                        Some(branch) => format!("{} · {branch}", group.name),
                        None => group.name.clone(),
                    };
                    return Row::new(vec![
                        Cell::from(if expanded { "▾" } else { "▸" }),
                        Cell::from(""),
                        Cell::from(if state.locale == crate::presentation::i18n::Locale::FR {
                            "PROJET"
                        } else {
                            "PROJECT"
                        }),
                        Cell::from(fit(&name, title_width)),
                        Cell::from(format!(
                            "{} {}",
                            group.endpoints.len(),
                            if group.endpoints.len() == 1 {
                                "port"
                            } else {
                                "ports"
                            }
                        )),
                    ])
                    .style(style(ACCENT).bg(RAISED).add_modifier(Modifier::BOLD));
                }
                VisibleRow::Endpoint(index) => *index,
            };
            let e = &state.snapshot.endpoints[index];
            let a = assess(&e.process);
            let text = state.locale.assessment(&a);
            let protected = blocked(&e.process, &state.actor).is_some();
            let action = if blocked(&e.process, &state.actor).is_some() {
                state.locale.text(T::Protected)
            } else if a.needs_pid {
                state.locale.text(T::Caution)
            } else {
                state.locale.text(T::Confirm)
            };
            Row::new(vec![
                Cell::from(format!("{}", e.port)).style(style(TEXT).add_modifier(Modifier::BOLD)),
                Cell::from(e.protocol.label()).style(style(SUBTLE)),
                Cell::from(text.badge).style(style(kind_color(a.kind))),
                Cell::from(fit(&a.title, title_width)),
                Cell::from(action).style(style(if protected {
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
        Row::new([
            "PORT",
            "NET",
            state.locale.text(T::Usage),
            state.locale.text(T::ProjectService),
            state.locale.text(T::Stop),
        ])
        .style(style(SUBTLE))
        .bottom_margin(1),
    )
    .row_highlight_style(style(TEXT).bg(SELECTED));
    frame.render_stateful_widget(table, inner, &mut state.table);
    if state.visible.is_empty() {
        frame.render_widget(
            Paragraph::new(if state.busy {
                state.locale.text(T::Reading)
            } else {
                state.locale.text(T::Empty)
            })
            .style(style(MUTED)),
            state.table_rect,
        );
    }
}
