use super::ports::kind_color;
use crate::domain::{assessment::assess, policy::blocked};
use crate::presentation::{design::*, state::State};
use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
pub fn render(frame: &mut Frame, state: &mut State, area: Rect) {
    let Some(e) = state.selected().cloned() else {
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style(BORDER))
                .title(" DÉTAIL "),
            area,
        );
        return;
    };
    let a = assess(&e.process);
    let p = &e.process;
    let policy = blocked(p, &state.actor);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style(if state.detail { ACCENT } else { BORDER }))
        .style(style(TEXT).bg(SURFACE))
        .title(Line::from(format!(" :{}  {} ", e.port, a.badge)).style(style(kind_color(a.kind))));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let footer_height = if inner.height >= 12 { 4 } else { 3 };
    let content = Rect {
        height: inner.height.saturating_sub(footer_height),
        ..inner
    };
    state.details_rect = content;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut add = |text: &str, color| {
        for text in wrap(text, content.width as usize) {
            lines.push(Line::from(text).style(style(color)));
        }
    };
    add(&a.title, TEXT);
    add(a.label, kind_color(a.kind));
    add("", TEXT);
    add(a.impact, WARNING);
    add("", TEXT);
    add(&a.reason, MUTED);
    add("", TEXT);
    add(&format!("PID {}   UID {}", p.pid, p.uid), TEXT);
    add(
        &format!(
            "CPU {:.1}%  ·  RAM {:.1} Mo",
            p.cpu,
            p.memory_kb as f64 / 1024.0
        ),
        MUTED,
    );
    for (label, value) in [
        ("DOSSIER", p.cwd.as_deref().unwrap_or("Indisponible")),
        (
            "EXÉCUTABLE",
            p.executable.as_deref().unwrap_or("Indisponible"),
        ),
        ("COMMANDE", &p.command),
        ("DÉMARRÉ", &p.started_at),
        ("ADRESSES", &e.addresses.join(", ")),
    ] {
        add("", TEXT);
        add(label, SUBTLE);
        add(value, TEXT);
    }
    state.detail_scroll = state
        .detail_scroll
        .min(lines.len().saturating_sub(content.height as usize) as u16);
    frame.render_widget(
        Paragraph::new(lines).scroll((state.detail_scroll, 0)),
        content,
    );
    let y = content.bottom();
    let ports = state
        .snapshot
        .endpoints
        .iter()
        .filter(|row| row.process.pid == p.pid)
        .map(|row| row.port.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    frame.render_widget(
        Paragraph::new(fit(
            &format!("Ports du PID : {ports}"),
            inner.width as usize,
        ))
        .style(style(SUBTLE)),
        Rect::new(inner.x, y, inner.width, 1),
    );
    let action_y = inner.bottom() - 1;
    if footer_height == 4 {
        frame.render_widget(
            Paragraph::new("↵ focus / retour · ↑↓ défiler").style(style(SUBTLE)),
            Rect::new(inner.x, y + 1, inner.width, 1),
        );
    }
    let action = policy.unwrap_or(if a.needs_pid {
        "k Arrêter · PID requis"
    } else {
        "k Arrêter · confirmation"
    });
    state.action_rect = if policy.is_none() {
        Rect::new(inner.x, action_y, inner.width, 1)
    } else {
        Rect::default()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            fit(action, inner.width as usize),
            style(if policy.is_some() { SUBTLE } else { WARNING })
                .bg(RAISED)
                .add_modifier(Modifier::BOLD),
        ))),
        Rect::new(inner.x, action_y, inner.width, 1),
    );
}
