use crate::presentation::{
    design::*,
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
        Overlay::Help => {
            let area = centered(frame.area(), 76, 19);
            frame.render_widget(Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(style(ACCENT))
                .style(style(TEXT).bg(RAISED))
                .title(" RACCOURCIS ");
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let help = [
                "0 Tous  1 Projets  2 Services  3 Apps  4 Système  5 Inconnus",
                "Tab          TCP + UDP / TCP / UDP",
                "↑↓ / PgUp/Dn Choisir un port",
                "/            Rechercher port, projet, commande ou PID",
                "Entrée       Ouvrir / quitter le détail",
                "↑↓ / molette Défiler le détail ou la confirmation",
                "k            Préparer l’arrêt normal (SIGTERM)",
                "K            Préparer l’arrêt forcé (SIGKILL)",
                "O            Confirmer un arrêt de projet",
                "PID + Entrée Confirmer un arrêt sensible",
                "Échap        Annuler / fermer / effacer le filtre",
                "Espace       Pause / reprise des relevés",
                "r            Actualiser maintenant",
                "q / Ctrl+C   Quitter",
                "",
                "Aucune escalade automatique. Arrêter vise un processus",
                "et tous ses ports. Échap / ? pour fermer cette aide.",
            ];
            frame.render_widget(
                Paragraph::new(
                    help.map(|s| Line::from(fit(s, inner.width as usize)))
                        .to_vec(),
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
                    " FORCER L’ARRÊT · SIGKILL "
                } else {
                    " CONFIRMER L’ARRÊT · SIGTERM "
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
                plan.assessment.impact.to_string(),
                if plan.force {
                    "SIGKILL ne laisse pas au processus le temps de nettoyer ou d’enregistrer son état.".into()
                } else {
                    "SIGTERM demande au processus de se fermer. Aucune escalade automatique.".into()
                },
                format!("Tous les ports concernés : {ports}"),
                format!("Commande : {}", p.command),
                format!("Dossier : {}", p.cwd.as_deref().unwrap_or("indisponible")),
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
                format!("Saisis le PID {} : {}▏", p.pid, input)
            } else {
                "O pour confirmer · Entrée n’envoie aucun signal".into()
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
                "Vérification et arrêt en cours…"
            } else if ready {
                if plan.assessment.needs_pid {
                    "Entrée · CONFIRMER L’ARRÊT"
                } else {
                    "O · CONFIRMER L’ARRÊT"
                }
            } else {
                "CONFIRMER · PID exact requis"
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
                Paragraph::new("Échap annuler · ↑↓ / molette pour tout lire").style(style(SUBTLE)),
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
