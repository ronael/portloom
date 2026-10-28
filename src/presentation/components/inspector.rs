use super::ports::kind_color;
use crate::domain::{assessment::assess, policy::blocked};
use crate::presentation::{design::*, grouping::VisibleRow, i18n::T, state::State};
use ratatui::{
    Frame,
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
pub fn render(frame: &mut Frame, state: &mut State, area: Rect) {
    if let Some(VisibleRow::Group(group)) = state.selected_row() {
        let processes = group
            .endpoints
            .iter()
            .map(|i| state.snapshot.endpoints[*i].process.pid)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let counts = state.locale.format(
            T::GroupCounts,
            &[
                ("ports", &group.endpoints.len().to_string()),
                ("processes", &processes.to_string()),
            ],
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(style(ACCENT))
            .style(style(TEXT).bg(SURFACE))
            .title(format!(" {} ", state.locale.text(T::Detail)));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let content = [
            group.name.clone(),
            counts,
            String::new(),
            group.root.clone(),
            String::new(),
            state.locale.text(T::GroupHint).into(),
        ];
        let lines = content
            .iter()
            .flat_map(|line| wrap(line, inner.width as usize))
            .map(Line::from)
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(lines), inner);
        return;
    }
    let Some(e) = state.selected().cloned() else {
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style(BORDER))
                .title(format!(" {} ", state.locale.text(T::Detail))),
            area,
        );
        return;
    };
    let a = assess(&e.process);
    let text = state.locale.assessment(&a);
    let p = &e.process;
    let policy = blocked(p, &state.actor);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style(if state.detail { ACCENT } else { BORDER }))
        .style(style(TEXT).bg(SURFACE))
        .title(
            Line::from(format!(" :{}  {} ", e.port, text.badge)).style(style(kind_color(a.kind))),
        );
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
    add(text.label, kind_color(a.kind));
    add("", TEXT);
    add(text.impact, WARNING);
    add("", TEXT);
    add(&text.reason, MUTED);
    add("", TEXT);
    add(&format!("PID {}   UID {}", p.pid, p.uid), TEXT);
    add(
        &format!(
            "CPU {:.1}%  ·  RAM {:.1} MiB",
            p.cpu,
            p.memory_kb as f64 / 1024.0
        ),
        MUTED,
    );
    for (label, value) in [
        (
            state.locale.text(T::Directory),
            p.cwd
                .as_deref()
                .unwrap_or(state.locale.text(T::Unavailable)),
        ),
        (
            state.locale.text(T::Executable),
            p.executable
                .as_deref()
                .unwrap_or(state.locale.text(T::Unavailable)),
        ),
        (state.locale.text(T::Command), &p.command),
        (state.locale.text(T::Started), &p.started_at),
        (state.locale.text(T::Addresses), &e.addresses.join(", ")),
    ] {
        add("", TEXT);
        add(label, SUBTLE);
        add(value, TEXT);
    }
    if let Some(branch) = p
        .project
        .as_ref()
        .and_then(|project| project.branch.as_deref())
    {
        add("", TEXT);
        add(state.locale.text(T::Branch), SUBTLE);
        add(branch, TEXT);
    }
    add("", TEXT);
    if let Some(report) = state.web_reports.get(&e.id) {
        add(state.locale.text(T::HttpLastCheck), SUBTLE);
        add(
            &format!(
                "HTTP {} · {} ms",
                report.response.status, report.response.elapsed_ms
            ),
            ACCENT,
        );
        add(&report.response.target.url(), MUTED);
    } else {
        add(state.locale.text(T::HttpUnchecked), SUBTLE);
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
            &state.locale.format(T::PidPorts, &[("ports", &ports)]),
            inner.width as usize,
        ))
        .style(style(SUBTLE)),
        Rect::new(inner.x, y, inner.width, 1),
    );
    let action_y = inner.bottom() - 1;
    if footer_height == 4 {
        frame.render_widget(
            Paragraph::new(state.locale.text(T::DetailHint)).style(style(SUBTLE)),
            Rect::new(inner.x, y + 1, inner.width, 1),
        );
    }
    frame.render_widget(
        Paragraph::new(fit(state.locale.text(T::HttpHint), inner.width as usize))
            .style(style(MUTED)),
        Rect::new(inner.x, action_y - 1, inner.width, 1),
    );
    let action = policy
        .as_ref()
        .map(|issue| state.locale.error(issue))
        .unwrap_or_else(|| {
            state
                .locale
                .text(if a.needs_pid {
                    T::StopPid
                } else {
                    T::StopConfirm
                })
                .into()
        });
    state.action_rect = if policy.is_none() {
        Rect::new(inner.x, action_y, inner.width, 1)
    } else {
        Rect::default()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            fit(&action, inner.width as usize),
            style(if policy.is_some() { SUBTLE } else { WARNING })
                .bg(RAISED)
                .add_modifier(Modifier::BOLD),
        ))),
        Rect::new(inner.x, action_y, inner.width, 1),
    );
}
