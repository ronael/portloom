use super::{
    components::{dialogs, inspector, ports},
    design::*,
    state::State,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Paragraph, Sparkline},
};
use std::collections::BTreeSet;
pub fn render(frame: &mut Frame, state: &mut State) {
    let area = frame.area();
    frame.render_widget(Block::default().style(style(TEXT).bg(CANVAS)), area);
    state.table_rect = Rect::default();
    state.action_rect = Rect::default();
    state.details_rect = Rect::default();
    state.dialog_action = Rect::default();
    state.too_small = area.width < 60 || area.height < 20;
    if state.too_small {
        frame.render_widget(
            Paragraph::new(
                "PORTLOOM\n\nAgrandis le terminal : 60 × 20 minimum.\nq / Ctrl+C pour quitter.",
            )
            .style(style(ACCENT)),
            area,
        );
        return;
    }
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);
    header(frame, state, layout[0]);
    toolbar(frame, state, layout[1]);
    if area.width >= 110 {
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(40), Constraint::Length(38)])
            .spacing(1)
            .split(layout[2]);
        ports::render(frame, state, panels[0]);
        inspector::render(frame, state, panels[1]);
    } else if state.detail {
        inspector::render(frame, state, layout[2]);
    } else {
        ports::render(frame, state, layout[2]);
    }
    frame.render_widget(
        Paragraph::new(fit(&state.notice, layout[3].width as usize)).style(style(if state.error {
            DANGER
        } else {
            MUTED
        })),
        layout[3],
    );
    let footer = if area.width >= 100 {
        "↑↓ choisir   / rechercher   Entrée détail   k arrêter   K forcer   espace pause   ? aide   q quitter"
    } else {
        "↑↓ choix  / chercher  ↵ détail  k arrêt  ? aide  q quitter"
    };
    frame.render_widget(
        Paragraph::new(fit(footer, layout[4].width as usize)).style(style(SUBTLE)),
        layout[4],
    );
    dialogs::render(frame, state);
}
fn header(frame: &mut Frame, state: &State, area: Rect) {
    let mode = if state.demo {
        "DÉMO · aucun effet système"
    } else if state.stale {
        "ERREUR · données anciennes"
    } else if state.paused {
        "PAUSE"
    } else if state.busy {
        "LECTURE…"
    } else {
        "LOCAL · EN DIRECT"
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("PORTLOOM", style(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("  /  ", style(BORDER)),
            Span::styled(mode, style(if state.stale { DANGER } else { MUTED })),
        ])),
        Rect { height: 1, ..area },
    );
    let endpoints = &state.snapshot.endpoints;
    let tcp = endpoints
        .iter()
        .filter(|e| e.protocol == crate::domain::model::Protocol::Tcp)
        .count();
    let count = endpoints
        .iter()
        .map(|e| e.process.pid)
        .collect::<BTreeSet<_>>()
        .len();
    let counts = format!(
        "{} ports   {tcp} TCP   {} UDP   {count} processus",
        endpoints.len(),
        endpoints.len() - tcp
    );
    frame.render_widget(
        Paragraph::new(counts).style(style(TEXT)),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
    if area.width >= 95 {
        frame.render_widget(
            Sparkline::default()
                .data(&state.history)
                .style(style(ACCENT)),
            Rect::new(area.right() - 22, area.y, 22, 2),
        );
    }
}
fn toolbar(frame: &mut Frame, state: &State, area: Rect) {
    let labels = if area.width >= 90 {
        [
            "0 Tous",
            "1 Projets",
            "2 Services",
            "3 Apps",
            "4 Système",
            "5 Inconnus",
        ]
    } else {
        ["0 Tout", "1 Proj", "2 Serv", "3 Apps", "4 Syst", "5 Inco"]
    };
    let spans = labels
        .into_iter()
        .enumerate()
        .flat_map(|(i, label)| {
            [
                Span::styled(
                    format!(" {label} "),
                    if state.view as usize == i {
                        style(CANVAS).bg(ACCENT).add_modifier(Modifier::BOLD)
                    } else {
                        style(MUTED).bg(SURFACE)
                    },
                ),
                Span::raw(" "),
            ]
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Line::from(spans)),
        Rect { height: 1, ..area },
    );
    let protocol = ["TCP + UDP", "TCP", "UDP"][state.protocol as usize];
    let query = if state.searching {
        format!("/ {}▏", safe(&state.query))
    } else if state.query.is_empty() {
        "/ rechercher un port, projet, PID…".into()
    } else {
        format!("/ {}", safe(&state.query))
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("Tab {protocol}  ·  "), style(SUBTLE)),
            Span::styled(query, style(if state.searching { ACCENT } else { MUTED })),
        ])),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
}
