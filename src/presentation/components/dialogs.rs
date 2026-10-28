use crate::presentation::{
    design::*,
    i18n::{Locale, T},
    state::{Overlay, State},
};
use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
pub fn render(frame: &mut Frame, state: &mut State) {
    match &mut state.overlay {
        Overlay::None => {}
        Overlay::Language { selected } => {
            let area = centered(frame.area(), 64, 13);
            frame.render_widget(Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(style(ACCENT))
                .style(style(TEXT).bg(RAISED))
                .title(" LANGUAGE / LANGUE ");
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let language = Locale(*selected);
            let lines = vec![
                Line::from("Choose your language / Choisis ta langue"),
                Line::from(""),
                Line::from("  1  English").style(if language == Locale::EN {
                    style(CANVAS).bg(ACCENT)
                } else {
                    style(MUTED)
                }),
                Line::from("  2  Français").style(if language == Locale::FR {
                    style(CANVAS).bg(ACCENT)
                } else {
                    style(MUTED)
                }),
                Line::from(""),
                Line::from("↑↓ select / choisir   Enter save / enregistrer"),
                Line::from("Esc: continue once / continuer sans enregistrer"),
                Line::from(""),
                Line::from("Change anytime: L / portloom language en|fr").style(style(SUBTLE)),
            ];
            frame.render_widget(Paragraph::new(lines), inner);
        }
        Overlay::Help => {
            let area = centered(frame.area(), 76, 19);
            frame.render_widget(Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(style(ACCENT))
                .style(style(TEXT).bg(RAISED))
                .title(state.locale.text(T::Shortcuts));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let help = state.locale.text(T::Help).lines();
            frame.render_widget(
                Paragraph::new(
                    help.map(|s| Line::from(fit(s, inner.width as usize)))
                        .collect::<Vec<_>>(),
                ),
                inner,
            );
        }
        Overlay::Confirm {
            plan,
            input,
            scroll,
        } => {
            let area = centered(frame.area(), 78, 25);
            let color = if plan.force { DANGER } else { WARNING };
            frame.render_widget(Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(style(color))
                .style(style(TEXT).bg(RAISED))
                .title(if plan.force {
                    state.locale.text(T::ForceTitle)
                } else {
                    state.locale.text(T::ConfirmTitle)
                });
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let p = &plan.target;
            frame.render_widget(
                Paragraph::new(fit(
                    &format!("{}  ·  PID {}", plan.assessment.title, p.pid),
                    inner.width as usize,
                ))
                .style(style(TEXT).add_modifier(Modifier::BOLD)),
                Rect { height: 1, ..inner },
            );
            let content = Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(7),
            );
            let ports = plan
                .endpoints
                .iter()
                .map(|e| format!("{}:{}", e.protocol.label(), e.port))
                .collect::<Vec<_>>()
                .join("  ");
            let paragraphs = [
                state.locale.assessment(&plan.assessment).impact.to_string(),
                if plan.force {
                    state.locale.text(T::KillWarning).into()
                } else {
                    state.locale.text(T::TermInfo).into()
                },
                state.locale.format(T::AffectedPorts, &[("ports", &ports)]),
                state
                    .locale
                    .format(T::CommandValue, &[("value", &p.command)]),
                state.locale.format(
                    T::DirectoryValue,
                    &[(
                        "value",
                        p.cwd
                            .as_deref()
                            .unwrap_or(state.locale.text(T::Unavailable)),
                    )],
                ),
            ];
            let lines: Vec<Line<'static>> = paragraphs
                .iter()
                .flat_map(|text| {
                    wrap(text, content.width as usize)
                        .into_iter()
                        .chain([String::new()])
                })
                .map(Line::from)
                .collect();
            *scroll = (*scroll).min(lines.len().saturating_sub(content.height as usize) as u16);
            frame.render_widget(
                Paragraph::new(lines)
                    .style(style(MUTED))
                    .scroll((*scroll, 0)),
                content,
            );
            let ready = !state.busy && (!plan.assessment.needs_pid || *input == p.pid.to_string());
            let prompt = if plan.assessment.needs_pid {
                state
                    .locale
                    .format(T::TypePid, &[("pid", &p.pid.to_string()), ("input", input)])
            } else {
                state.locale.text(T::TypeConfirm).into()
            };
            frame.render_widget(
                Paragraph::new(fit(&prompt, inner.width as usize)).style(style(if ready {
                    ACCENT
                } else {
                    color
                })),
                Rect::new(inner.x, inner.bottom() - 4, inner.width, 1),
            );
            state.dialog_action = Rect::new(inner.x, inner.bottom() - 2, inner.width, 1);
            let action = if state.busy {
                state.locale.text(T::Stopping)
            } else if ready {
                if plan.assessment.needs_pid {
                    state.locale.text(T::EnterConfirm)
                } else {
                    state.locale.text(T::LetterConfirm)
                }
            } else {
                state.locale.text(T::ExactPid)
            };
            frame.render_widget(
                Paragraph::new(action).style(
                    style(if ready { CANVAS } else { SUBTLE }).bg(if ready {
                        color
                    } else {
                        SURFACE
                    }),
                ),
                state.dialog_action,
            );
            frame.render_widget(
                Paragraph::new(state.locale.text(T::CancelHint)).style(style(SUBTLE)),
                Rect::new(inner.x, inner.bottom() - 1, inner.width, 1),
            );
        }
    }
}
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(4));
    let height = height.min(area.height.saturating_sub(2));
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}
