use super::{
    state::{Action, Overlay, State},
    view,
};
use crate::{
    application::{
        ports::{AppResult, PortScanner},
        stop::{StopPlan, StopProcess, StopResult},
    },
    domain::{
        model::{Endpoint, Snapshot},
        policy::Actor,
    },
};
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyEventKind, MouseButton, MouseEventKind,
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
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};
enum Job {
    Scan,
    Prepare(Box<Endpoint>, bool),
    Execute(Box<StopPlan>, String),
}
enum Reply {
    Scan(AppResult<Snapshot>),
    Prepare(AppResult<Box<StopPlan>>),
    Execute(AppResult<StopResult>),
}
fn worker(
    scanner: Arc<dyn PortScanner>,
    stopper: Arc<StopProcess>,
) -> (Sender<Job>, Receiver<Reply>) {
    let (send, jobs) = mpsc::channel();
    let (replies, recv) = mpsc::channel();
    thread::spawn(move || {
        while let Ok(job) = jobs.recv() {
            let reply = match job {
                Job::Scan => Reply::Scan(scanner.scan()),
                Job::Prepare(e, force) => Reply::Prepare(stopper.prepare(&e, force).map(Box::new)),
                Job::Execute(plan, ack) => Reply::Execute(stopper.execute(&plan, &ack)),
            };
            if replies.send(reply).is_err() {
                break;
            }
        }
    });
    (send, recv)
}
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
pub fn run(
    scanner: Arc<dyn PortScanner>,
    stopper: Arc<StopProcess>,
    actor: Actor,
    demo: bool,
    interval: u64,
) -> AppResult<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(
            "L’interface exige un terminal interactif. Utilise --json dans un script.".into(),
        );
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
    let result = event_loop(scanner, stopper, actor, demo, interval, quit);
    for id in registrations {
        signal_hook::low_level::unregister(id);
    }
    result.map_err(|e| e.to_string())
}
fn event_loop(
    scanner: Arc<dyn PortScanner>,
    stopper: Arc<StopProcess>,
    actor: Actor,
    demo: bool,
    interval: u64,
    quit: Arc<AtomicBool>,
) -> io::Result<()> {
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let (jobs, replies) = worker(scanner, stopper);
    let mut state = State::new(actor, demo);
    let mut last_scan = Instant::now();
    let mut refresh = true;
    let mut dirty = true;
    while !quit.load(Ordering::Relaxed) {
        while let Ok(reply) = replies.try_recv() {
            state.busy = false;
            dirty = true;
            match reply {
                Reply::Scan(Ok(snapshot)) => {
                    state.apply(snapshot);
                    if state.notice == "Lecture des ports…" {
                        state.message("Sélectionne un port pour comprendre son usage.", false);
                    }
                    last_scan = Instant::now();
                }
                Reply::Scan(Err(e)) => {
                    state.stale = true;
                    state.message(e, true);
                    last_scan = Instant::now();
                }
                Reply::Prepare(Ok(plan)) => {
                    state.overlay = Overlay::Confirm {
                        plan,
                        input: String::new(),
                        scroll: 0,
                    };
                    state.message("Vérifie les ports concernés avant de confirmer.", false);
                }
                Reply::Prepare(Err(e)) => state.message(e, true),
                Reply::Execute(result) => {
                    state.overlay = Overlay::None;
                    match result {
                        Ok(r) => state.stop_result(r),
                        Err(e) => state.message(e, true),
                    };
                    refresh = true;
                }
            }
        }
        if !state.busy
            && matches!(state.overlay, Overlay::None)
            && (refresh
                || (!state.paused && last_scan.elapsed() >= Duration::from_millis(interval)))
        {
            jobs.send(Job::Scan).map_err(io::Error::other)?;
            state.busy = true;
            refresh = false;
            dirty = true;
        }
        if dirty {
            terminal.draw(|frame| view::render(frame, &mut state))?;
            dirty = false;
        }
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let action = match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
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
                        if index < state.visible.len() {
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
            Action::Quit => break,
            Action::Refresh => {
                refresh = true;
                state.message("Actualisation demandée.", false);
            }
            Action::Prepare(e, force) => {
                state.busy = true;
                state.message("Vérification du processus…", false);
                jobs.send(Job::Prepare(e, force))
                    .map_err(io::Error::other)?;
            }
            Action::Execute(plan, ack) => {
                state.busy = true;
                state.message("Vérification puis envoi du signal…", false);
                jobs.send(Job::Execute(plan, ack))
                    .map_err(io::Error::other)?;
            }
            Action::None => {}
        }
    }
    Ok(())
}
