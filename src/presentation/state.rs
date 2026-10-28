use crate::{
    application::stop::{StopPlan, StopResult},
    domain::{
        assessment::assess,
        model::{Endpoint, Protocol, Snapshot},
        policy::{Actor, blocked},
    },
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, widgets::TableState};
#[derive(Default)]
pub enum Overlay {
    #[default]
    None,
    Help,
    Confirm {
        plan: Box<StopPlan>,
        input: String,
        scroll: u16,
    },
}
pub enum Action {
    None,
    Quit,
    Refresh,
    Prepare(Box<Endpoint>, bool),
    Execute(Box<StopPlan>, String),
}
pub struct State {
    pub snapshot: Snapshot,
    pub actor: Actor,
    pub demo: bool,
    pub view: u8,
    pub protocol: u8,
    pub query: String,
    pub searching: bool,
    pub paused: bool,
    pub busy: bool,
    pub detail: bool,
    pub detail_scroll: u16,
    pub overlay: Overlay,
    pub visible: Vec<usize>,
    pub table: TableState,
    pub notice: String,
    pub error: bool,
    pub too_small: bool,
    pub stale: bool,
    pub history: Vec<u64>,
    pub table_rect: Rect,
    pub action_rect: Rect,
    pub dialog_action: Rect,
    pub details_rect: Rect,
    pub page_size: usize,
}
impl State {
    pub fn new(actor: Actor, demo: bool) -> Self {
        Self {
            snapshot: Snapshot::default(),
            actor,
            demo,
            view: 0,
            protocol: 0,
            query: String::new(),
            searching: false,
            paused: false,
            busy: false,
            detail: false,
            detail_scroll: 0,
            overlay: Overlay::None,
            visible: vec![],
            table: TableState::default(),
            notice: "Lecture des ports…".into(),
            error: false,
            too_small: false,
            stale: false,
            history: vec![],
            table_rect: Rect::default(),
            action_rect: Rect::default(),
            dialog_action: Rect::default(),
            details_rect: Rect::default(),
            page_size: 10,
        }
    }
    pub fn selected(&self) -> Option<&Endpoint> {
        self.table
            .selected()
            .and_then(|i| self.visible.get(i))
            .and_then(|i| self.snapshot.endpoints.get(*i))
    }
    pub fn apply(&mut self, snapshot: Snapshot) {
        let selected = self.selected().map(|e| e.id.clone());
        self.history.push(snapshot.endpoints.len() as u64);
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.snapshot = snapshot;
        self.stale = false;
        self.rebuild();
        if let Some(id) = selected
            && let Some(index) = self
                .visible
                .iter()
                .position(|i| self.snapshot.endpoints[*i].id == id)
        {
            self.table.select(Some(index));
        }
    }
    pub fn rebuild(&mut self) {
        let q = self.query.to_lowercase();
        self.visible = self
            .snapshot
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let a = assess(&e.process);
                a.kind.in_view(self.view)
                    && (self.protocol == 0
                        || (self.protocol == 1 && e.protocol == Protocol::Tcp)
                        || (self.protocol == 2 && e.protocol == Protocol::Udp))
                    && (q.is_empty()
                        || format!(
                            "{} {} {} {} {} {} {} {} {}",
                            e.port,
                            e.process.pid,
                            e.protocol.label(),
                            e.process.name,
                            e.process.command,
                            e.process.cwd.as_deref().unwrap_or(""),
                            a.title,
                            a.label,
                            e.addresses.join(" ")
                        )
                        .to_lowercase()
                        .contains(&q))
            })
            .map(|(i, _)| i)
            .collect();
        self.visible.sort_by_key(|i| {
            let e = &self.snapshot.endpoints[*i];
            (assess(&e.process).kind, e.port, e.process.pid)
        });
        self.table.select(if self.visible.is_empty() {
            None
        } else {
            Some(
                self.table
                    .selected()
                    .unwrap_or(0)
                    .min(self.visible.len() - 1),
            )
        });
        self.detail_scroll = 0;
    }
    pub fn move_selection(&mut self, delta: isize) {
        if self.visible.is_empty() {
            return;
        }
        let current = self.table.selected().unwrap_or(0);
        let next = current
            .saturating_add_signed(delta)
            .min(self.visible.len() - 1);
        self.table.select(Some(next));
        self.detail_scroll = 0;
    }
    pub fn message(&mut self, text: impl Into<String>, error: bool) {
        self.notice = text.into();
        self.error = error;
    }
    pub fn stop_result(&mut self, result: StopResult) {
        self.message(
            match result {
                StopResult::Stopped => {
                    if self.demo {
                        "Simulation : processus arrêté."
                    } else {
                        "Processus arrêté."
                    }
                }
                StopResult::Gone => "Le processus ou le port a déjà disparu.",
                StopResult::Running => {
                    "Signal envoyé, processus toujours présent. Aucun arrêt forcé automatique."
                }
            },
            result == StopResult::Running,
        );
    }
    pub fn prepare(&mut self, force: bool) -> Action {
        if self.busy {
            self.message("Lecture en cours, réessaie dans un instant.", false);
            return Action::None;
        }
        if let Some(e) = self.selected() {
            if let Some(reason) = blocked(&e.process, &self.actor) {
                self.message(reason, true);
            } else {
                return Action::Prepare(Box::new(e.clone()), force);
            }
        }
        Action::None
    }
    pub fn confirm(&mut self) -> Action {
        if self.busy {
            return Action::None;
        }
        if let Overlay::Confirm { plan, input, .. } = &self.overlay
            && (!plan.assessment.needs_pid || *input == plan.target.pid.to_string())
        {
            return Action::Execute(plan.clone(), input.clone());
        }
        Action::None
    }
    pub fn key(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }
        if self.too_small {
            return if key.code == KeyCode::Char('q') {
                Action::Quit
            } else {
                Action::None
            };
        }
        if let Overlay::Confirm {
            plan,
            input,
            scroll,
        } = &mut self.overlay
        {
            match key.code {
                KeyCode::Esc if !self.busy => {
                    self.overlay = Overlay::None;
                    self.message("Arrêt annulé.", false);
                }
                KeyCode::Char(c)
                    if plan.assessment.needs_pid && c.is_ascii_digit() && input.len() < 10 =>
                {
                    input.push(c)
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Down | KeyCode::PageDown => *scroll = scroll.saturating_add(3),
                KeyCode::Up | KeyCode::PageUp => *scroll = scroll.saturating_sub(3),
                KeyCode::Enter if plan.assessment.needs_pid => return self.confirm(),
                KeyCode::Char('o' | 'O') if !plan.assessment.needs_pid => return self.confirm(),
                _ => {}
            }
            return Action::None;
        }
        if matches!(self.overlay, Overlay::Help) {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
            ) {
                self.overlay = Overlay::None;
            }
            return Action::None;
        }
        if self.searching {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.searching = false,
                KeyCode::Backspace => {
                    self.query.pop();
                    self.rebuild();
                }
                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    if self.query.len() < 256 {
                        self.query.push(c);
                        self.rebuild();
                    }
                }
                _ => {}
            }
            return Action::None;
        }
        match key.code {
            KeyCode::Char('q') => return Action::Quit,
            KeyCode::Char('/') => self.searching = true,
            KeyCode::Esc => {
                self.query.clear();
                self.detail = false;
                self.rebuild();
            }
            KeyCode::Char('0'..='5') => {
                if let KeyCode::Char(c) = key.code {
                    self.view = c as u8 - b'0';
                    self.rebuild();
                }
            }
            KeyCode::Tab => {
                self.protocol = (self.protocol + 1) % 3;
                self.rebuild();
            }
            KeyCode::Char(' ') => {
                self.paused = !self.paused;
                self.message(
                    if self.paused {
                        "Rafraîchissement en pause."
                    } else {
                        "Rafraîchissement repris."
                    },
                    false,
                );
            }
            KeyCode::Char('r') => return Action::Refresh,
            KeyCode::Char('k') => return self.prepare(false),
            KeyCode::Char('K') => return self.prepare(true),
            KeyCode::Enter => {
                self.detail = !self.detail;
                self.detail_scroll = 0;
            }
            KeyCode::Char('?') => self.overlay = Overlay::Help,
            KeyCode::Down | KeyCode::Char('j') if self.detail => {
                self.detail_scroll = self.detail_scroll.saturating_add(1)
            }
            KeyCode::Up if self.detail => self.detail_scroll = self.detail_scroll.saturating_sub(1),
            KeyCode::PageDown if self.detail => {
                self.detail_scroll = self.detail_scroll.saturating_add(8)
            }
            KeyCode::PageUp if self.detail => {
                self.detail_scroll = self.detail_scroll.saturating_sub(8)
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(self.page_size as isize),
            KeyCode::PageUp => self.move_selection(-(self.page_size as isize)),
            KeyCode::Home => {
                self.table.select(Some(0));
            }
            KeyCode::End => self.table.select(self.visible.len().checked_sub(1)),
            _ => {}
        }
        Action::None
    }
}
