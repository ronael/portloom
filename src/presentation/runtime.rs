use super::{
    animation::{AvatarFrame, Sequence},
    components::mascot,
    i18n::{Locale, T},
    state::{Action, Overlay, State},
    tasks::{Job, Reply, Services, worker},
    view,
};
use crate::{
    application::{ports::AppResult, preferences::Language},
    domain::policy::Actor,
};
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, layout::Position};
use std::{
    io::{self, IsTerminal},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            DisableMouseCapture,
            DisableBracketedPaste,
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        let _ = disable_raw_mode();
    }
}
pub struct Options {
    pub actor: Actor,
    pub demo: bool,
    pub interval: u64,
    pub locale: Locale,
    pub first_language: Option<Language>,
    pub port: Option<u16>,
    pub motion: bool,
}
pub fn run(services: Services, options: Options) -> AppResult<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(options.locale.text(T::TtyRequired).into());
    }
    let quit = Arc::new(AtomicBool::new(false));
    let mut registrations = Vec::new();
    for signal in [
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGHUP,
    ] {
        registrations
            .push(signal_hook::flag::register(signal, quit.clone()).map_err(|e| e.to_string())?);
    }
    let result = event_loop(services, options, quit);
    for id in registrations {
        signal_hook::low_level::unregister(id);
    }
    result.map_err(|e| e.to_string().into())
}
fn event_loop(services: Services, options: Options, quit: Arc<AtomicBool>) -> io::Result<()> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let (jobs, replies) = worker(services);
    let mut state = State::new(options.actor, options.demo);
    state.locale = options.locale;
    state.notice = state.locale.text(T::Reading).into();
    state.fixed_port = options.port;
    state.motion_enabled = options.motion;
    if let Some(selected) = options.first_language {
        state.overlay = Overlay::Language { selected };
    }
    let mut last_scan = Instant::now();
    let mut refresh = true;
    let mut dirty = true;
    let animation_start = Instant::now();
    while !quit.load(Ordering::Relaxed) {
        while let Ok(reply) = replies.try_recv() {
            state.busy = false;
            dirty = true;
            match reply {
                Reply::Scan(Ok(snapshot)) => {
                    state.apply(snapshot);
                    if state.notice == state.locale.text(T::Reading) {
                        state.message(state.locale.text(T::SelectPort), false);
                    }
                    last_scan = Instant::now();
                }
                Reply::Scan(Err(e)) => {
                    state.stale = true;
                    state.message(state.locale.error(&e), true);
                    last_scan = Instant::now();
                }
                Reply::Prepare(Ok(plan)) => {
                    state.overlay = Overlay::Confirm {
                        plan,
                        input: String::new(),
                        scroll: 0,
                    };
                    state.message(state.locale.text(T::ReviewPorts), false);
                }
                Reply::Prepare(Err(e)) => state.message(state.locale.error(&e), true),
                Reply::Web(result) => match result {
                    Ok(report) => state.web_result(*report),
                    Err(e) => state.message(state.locale.error(&e), true),
                },
                Reply::Language(language, result) => {
                    state.set_language(language);
                    if let Err(e) = result {
                        state.message(
                            format!(
                                "{} {}",
                                state.locale.text(T::LanguageSession),
                                state.locale.error(&e)
                            ),
                            true,
                        );
                    }
                }
                Reply::Execute(result) => {
                    state.overlay = Overlay::None;
                    match result {
                        Ok(r) => state.stop_result(r),
                        Err(e) => state.message(state.locale.error(&e), true),
                    };
                    refresh = true;
                }
            }
        }
        if !state.busy
            && matches!(state.overlay, Overlay::None)
            && (refresh
                || (!state.paused
                    && last_scan.elapsed() >= Duration::from_millis(options.interval)))
        {
            jobs.send(Job::Scan).map_err(io::Error::other)?;
            state.busy = true;
            refresh = false;
            dirty = true;
        }
        dirty |= state.animate(animation_start.elapsed().as_millis() as u64);
        if dirty {
            terminal.draw(|frame| view::render(frame, &mut state))?;
            dirty = false;
        }
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let action = match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                dirty = true;
                state.key(key)
            }
            Event::Resize(_, _) => {
                dirty = true;
                Action::None
            }
            Event::Paste(text) if state.searching => {
                for c in text
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(256usize.saturating_sub(state.query.len()))
                {
                    state.query.push(c);
                }
                state.rebuild();
                dirty = true;
                Action::None
            }
            Event::Mouse(mouse) => {
                dirty = true;
                let at = Position::new(mouse.column, mouse.row);
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left)
                        if matches!(state.overlay, Overlay::Confirm { .. })
                            && state.dialog_action.contains(at) =>
                    {
                        state.confirm()
                    }
                    MouseEventKind::ScrollDown
                        if matches!(state.overlay, Overlay::Confirm { .. }) =>
                    {
                        if let Overlay::Confirm { scroll, .. } = &mut state.overlay {
                            *scroll = scroll.saturating_add(3);
                        }
                        Action::None
                    }
                    MouseEventKind::ScrollUp
                        if matches!(state.overlay, Overlay::Confirm { .. }) =>
                    {
                        if let Overlay::Confirm { scroll, .. } = &mut state.overlay {
                            *scroll = scroll.saturating_sub(3);
                        }
                        Action::None
                    }
                    _ if !matches!(state.overlay, Overlay::None) => Action::None,
                    MouseEventKind::Down(MouseButton::Left) if state.action_rect.contains(at) => {
                        state.prepare(false)
                    }
                    MouseEventKind::Down(MouseButton::Left) if state.table_rect.contains(at) => {
                        let index = state.table.offset() + usize::from(at.y - state.table_rect.y);
                        if index < state.rows.len() {
                            state.table.select(Some(index));
                            state.detail_scroll = 0;
                        }
                        Action::None
                    }
                    MouseEventKind::ScrollDown if state.details_rect.contains(at) => {
                        state.detail_scroll = state.detail_scroll.saturating_add(3);
                        Action::None
                    }
                    MouseEventKind::ScrollUp if state.details_rect.contains(at) => {
                        state.detail_scroll = state.detail_scroll.saturating_sub(3);
                        Action::None
                    }
                    MouseEventKind::ScrollDown => {
                        state.move_selection(3);
                        Action::None
                    }
                    MouseEventKind::ScrollUp => {
                        state.move_selection(-3);
                        Action::None
                    }
                    _ => Action::None,
                }
            }
            _ => Action::None,
        };
        match action {
            Action::Quit => {
                if state.motion_enabled
                    && !state.too_small
                    && terminal.size()?.height >= mascot::MIN_TERMINAL_HEIGHT
                {
                    farewell(&mut terminal, &mut state, &quit)?;
                }
                break;
            }
            Action::Refresh => {
                refresh = true;
                state.message(state.locale.text(T::RefreshRequested), false);
            }
            Action::Prepare(e, force) => {
                state.busy = true;
                state.message(state.locale.text(T::CheckingProcess), false);
                jobs.send(Job::Prepare(e, force))
                    .map_err(io::Error::other)?;
            }
            Action::Execute(plan, ack) => {
                state.busy = true;
                state.message(state.locale.text(T::SendingSignal), false);
                jobs.send(Job::Execute(plan, ack))
                    .map_err(io::Error::other)?;
            }
            Action::Web(endpoint, open) => {
                state.busy = true;
                state.message(
                    state.locale.text(if open {
                        T::HttpOpening
                    } else {
                        T::HttpChecking
                    }),
                    false,
                );
                jobs.send(Job::Web(endpoint, open))
                    .map_err(io::Error::other)?;
            }
            Action::Language(language) => {
                state.busy = true;
                state.set_language(language);
                jobs.send(Job::Language(language))
                    .map_err(io::Error::other)?;
            }
            Action::None => {}
        }
    }
    Ok(())
}

fn farewell(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut State,
    quit: &AtomicBool,
) -> io::Result<()> {
    let started = Instant::now();
    while !quit.load(Ordering::Relaxed) {
        let elapsed = started.elapsed().as_millis() as u64;
        if elapsed >= Sequence::Departure.duration() {
            break;
        }
        let next = AvatarFrame::departing(elapsed);
        if next != state.avatar_frame {
            state.avatar_frame = next;
            terminal.draw(|frame| view::render(frame, state))?;
        }
        // Any key skips the farewell; OS signals also remain immediately actionable.
        if event::poll(Duration::from_millis(25))? {
            match event::read()? {
                Event::Key(_) => break,
                Event::Resize(_, _) => {
                    terminal.draw(|frame| view::render(frame, state))?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}
