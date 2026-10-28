use super::{
    grouping::{self, VisibleRow},
    i18n::{Locale, T},
};
use crate::application::{preferences::Language, web::WebReport};
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
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
pub enum Overlay {
    #[default]
    None,
    Help,
    Language {
        selected: Language,
    },
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
    Web(Box<Endpoint>, bool),
    Language(Language),
}
pub struct State {
    pub snapshot: Snapshot,
    pub locale: Locale,
    pub grouped: bool,
    pub collapsed: BTreeSet<String>,
    pub rows: Vec<VisibleRow>,
    pub fixed_port: Option<u16>,
    pub web_reports: BTreeMap<String, WebReport>,
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
            locale: Locale::EN,
            grouped: false,
            collapsed: BTreeSet::new(),
            rows: vec![],
            fixed_port: None,
            web_reports: BTreeMap::new(),
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
            notice: Locale::EN.text(T::Reading).into(),
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
    pub fn selected_row(&self) -> Option<&VisibleRow> {
        self.table.selected().and_then(|i| self.rows.get(i))
    }
    pub fn selected(&self) -> Option<&Endpoint> {
        match self.selected_row()? {
            VisibleRow::Endpoint(index) => self.snapshot.endpoints.get(*index),
            VisibleRow::Group(_) => None,
        }
    }
    fn selection_key(&self) -> Option<String> {
        self.selected_row().map(|row| row.key(&self.snapshot))
    }
    fn restore_selection(&mut self, key: Option<String>) {
        if let Some(key) = key
            && let Some(index) = self
                .rows
                .iter()
                .position(|row| row.key(&self.snapshot) == key)
        {
            self.table.select(Some(index));
        }
    }
    pub fn apply(&mut self, snapshot: Snapshot) {
        let selected = self.selection_key();
        self.history.push(snapshot.endpoints.len() as u64);
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        self.web_reports.retain(|id, report| {
            snapshot.endpoints.iter().any(|e| {
                e.id == *id
                    && crate::domain::model::same_process(&e.process, &report.endpoint.process)
                    && e.addresses == report.endpoint.addresses
            })
        });
        self.snapshot = snapshot;
        self.rows.clear();
        self.stale = false;
        self.rebuild();
        self.restore_selection(selected);
    }
    pub fn rebuild(&mut self) {
        let selection = self.selection_key();
        let q = self.query.to_lowercase();
        self.visible = self
            .snapshot
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                let a = assess(&e.process);
                a.kind.in_view(self.view)
                    && self.fixed_port.is_none_or(|port| port == e.port)
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
                            self.locale.assessment(&a).label,
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
        self.rows = grouping::rows(
            &self.snapshot,
            &self.visible,
            self.grouped,
            &self.collapsed,
            !q.is_empty(),
        );
        self.table.select(if self.rows.is_empty() {
            None
        } else {
            Some(self.table.selected().unwrap_or(0).min(self.rows.len() - 1))
        });
        self.restore_selection(selection);
        self.detail_scroll = 0;
    }
    pub fn move_selection(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let current = self.table.selected().unwrap_or(0);
        let next = current
            .saturating_add_signed(delta)
            .min(self.rows.len() - 1);
        self.table.select(Some(next));
        self.detail_scroll = 0;
    }
    pub fn message(&mut self, text: impl Into<String>, error: bool) {
        self.notice = text.into();
        self.error = error;
    }
    pub fn stop_result(&mut self, result: StopResult) {
        let key = match result {
            StopResult::Stopped if self.demo => T::DemoStopped,
            StopResult::Stopped => T::Stopped,
            StopResult::Gone => T::Gone,
            StopResult::Running => T::StillRunning,
        };
        self.message(self.locale.text(key), result == StopResult::Running);
    }
    pub fn set_language(&mut self, language: Language) {
        self.locale = Locale(language);
        self.overlay = Overlay::None;
        self.message(self.locale.text(T::LanguageSaved), false);
        self.rebuild();
    }
    pub fn web(&mut self, open: bool) -> Action {
        if self.busy {
            self.message(self.locale.text(T::Busy), false);
            return Action::None;
        }
        if let Some(endpoint) = self.selected().cloned() {
            self.web_reports.remove(&endpoint.id);
            return Action::Web(Box::new(endpoint), open);
        }
        self.message(self.locale.text(T::GroupHint), false);
        Action::None
    }
    pub fn web_result(&mut self, report: WebReport) {
        let response = &report.response;
        let key = if report.opened {
            if response.simulated {
                T::DemoOpened
            } else {
                T::Opened
            }
        } else {
            T::HttpResult
        };
        self.message(
            self.locale.format(
                key,
                &[
                    ("url", &response.target.url()),
                    ("status", &response.status.to_string()),
                    ("ms", &response.elapsed_ms.to_string()),
                ],
            ),
            false,
        );
        self.web_reports.insert(report.endpoint.id.clone(), report);
    }
    pub fn prepare(&mut self, force: bool) -> Action {
        if self.busy {
            self.message(self.locale.text(T::Busy), false);
            return Action::None;
        }
        if let Some(e) = self.selected() {
            if let Some(reason) = blocked(&e.process, &self.actor) {
                self.message(self.locale.error(&reason), true);
            } else {
                return Action::Prepare(Box::new(e.clone()), force);
            }
        }
        if self.selected().is_none() {
            self.message(self.locale.text(T::GroupHint), false);
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
        if let Overlay::Language { selected } = &mut self.overlay {
            match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => {
                    *selected = if *selected == Language::English {
                        Language::French
                    } else {
                        Language::English
                    };
                }
                KeyCode::Char('1') => *selected = Language::English,
                KeyCode::Char('2') => *selected = Language::French,
                KeyCode::Enter if !self.busy => return Action::Language(*selected),
                KeyCode::Esc => self.overlay = Overlay::None,
                _ => {}
            }
            return Action::None;
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
                    self.message(self.locale.text(T::Cancelled), false);
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
                KeyCode::Char(c)
                    if !plan.assessment.needs_pid
                        && c.to_ascii_lowercase() == self.locale.confirm_key() =>
                {
                    return self.confirm();
                }
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
                    if self.view == 1 {
                        self.grouped = true;
                    }
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
                        self.locale.text(T::Paused)
                    } else {
                        self.locale.text(T::Resumed)
                    },
                    false,
                );
            }
            KeyCode::Char('g') => {
                self.grouped = !self.grouped;
                self.rebuild();
                self.message(
                    self.locale.text(if self.grouped {
                        T::GroupToggle
                    } else {
                        T::FlatToggle
                    }),
                    false,
                );
            }
            KeyCode::Char('w') => return self.web(false),
            KeyCode::Char('o') => return self.web(true),
            KeyCode::Char('L') => {
                self.overlay = Overlay::Language {
                    selected: self.locale.0,
                }
            }
            KeyCode::Char('r') => return Action::Refresh,
            KeyCode::Char('k') => return self.prepare(false),
            KeyCode::Char('K') => return self.prepare(true),
            KeyCode::Enter => {
                if let Some(VisibleRow::Group(group)) = self.selected_row() {
                    let root = group.root.clone();
                    if !self.collapsed.remove(&root) {
                        if self.collapsed.len() >= 256 {
                            self.collapsed.pop_first();
                        }
                        self.collapsed.insert(root);
                    }
                    self.detail = false;
                    self.rebuild();
                } else {
                    self.detail = !self.detail;
                    self.detail_scroll = 0;
                }
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
            KeyCode::End => self.table.select(self.rows.len().checked_sub(1)),
            _ => {}
        }
        Action::None
    }
}
