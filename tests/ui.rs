use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portloom::{
    application::{ports::PortScanner, stop::StopProcess},
    domain::policy::Actor,
    infrastructure::{demo::DemoSystem, system::local::clock},
    presentation::{
        design::{fit, safe},
        state::{Action, Overlay, State},
        view,
    },
};
use ratatui::{Terminal, backend::TestBackend};
use std::sync::Arc;
fn state() -> State {
    let mut s = State::new(
        Actor {
            uid: 501,
            protected_pids: vec![],
        },
        true,
    );
    s.locale = portloom::presentation::i18n::Locale::FR;
    s.apply(DemoSystem::default().scan().unwrap());
    s
}
fn key(s: &mut State, c: KeyCode) -> Action {
    s.key(KeyEvent::new(c, KeyModifiers::NONE))
}
fn render(s: &mut State, w: u16, h: u16) -> String {
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| view::render(f, s)).unwrap();
    let b = term.backend().buffer();
    (0..h)
        .map(|y| (0..w).map(|x| b[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}
fn confirm(s: &mut State, sensitive: bool) {
    let demo = Arc::new(DemoSystem::default());
    let rows = demo.scan().unwrap().endpoints;
    let e = &rows[if sensitive { 2 } else { 0 }];
    let stop = StopProcess::new(demo.clone(), demo, s.actor.clone(), clock());
    s.overlay = Overlay::Confirm {
        plan: Box::new(stop.prepare(e, false).unwrap()),
        input: String::new(),
        scroll: 0,
    };
}
#[test]
fn wide_and_compact_layouts_keep_categories_actions_and_shortcuts() {
    for (w, h) in [(140, 40), (80, 24), (60, 20)] {
        let mut s = state();
        let text = render(&mut s, w, h);
        assert!(text.contains("PORTLOOM"));
        assert!(text.contains("SITE/API"));
        assert!(text.contains("CONFIRMER"));
        assert!(text.contains("q quitter"));
        if w >= 110 {
            assert!(text.contains("Ports du PID"));
        }
    }
}
#[test]
fn compact_inspector_remains_scrollable_and_action_visible() {
    let mut s = state();
    key(&mut s, KeyCode::Enter);
    let text = render(&mut s, 60, 20);
    assert!(text.contains("k Arrêter"));
    let mut saw_addresses = false;
    let mut saw_http = false;
    for _ in 0..50 {
        key(&mut s, KeyCode::Down);
        let text = render(&mut s, 60, 20);
        saw_addresses |= text.contains("ADRESSES");
        saw_http |= text.contains("HTTP non vérifié");
        assert!(text.contains("k Arrêter"));
    }
    assert!(
        saw_addresses && saw_http,
        "Every detail must be reachable while the stop action remains visible"
    );
}
#[test]
fn filter_search_focus_does_not_trigger_destructive_shortcuts() {
    let mut s = state();
    key(&mut s, KeyCode::Char('/'));
    for c in "kq5".chars() {
        assert!(matches!(key(&mut s, KeyCode::Char(c)), Action::None));
    }
    assert_eq!(s.query, "kq5");
    assert_eq!(s.view, 0);
    key(&mut s, KeyCode::Esc);
    key(&mut s, KeyCode::Esc);
    key(&mut s, KeyCode::Char('1'));
    assert_eq!(s.visible.len(), 4);
    key(&mut s, KeyCode::Char('4'));
    assert_eq!(s.visible.len(), 2);
    assert!(matches!(key(&mut s, KeyCode::Char('k')), Action::None));
    assert!(s.error);
}
#[test]
fn normal_dialog_requires_o_and_sensitive_dialog_requires_exact_pid() {
    let mut s = state();
    confirm(&mut s, false);
    assert!(matches!(key(&mut s, KeyCode::Enter), Action::None));
    assert!(matches!(
        key(&mut s, KeyCode::Char('o')),
        Action::Execute(..)
    ));
    confirm(&mut s, true);
    assert!(matches!(key(&mut s, KeyCode::Char('o')), Action::None));
    assert!(matches!(key(&mut s, KeyCode::Enter), Action::None));
    for c in "1206".chars() {
        key(&mut s, KeyCode::Char(c));
    }
    assert!(matches!(key(&mut s, KeyCode::Enter), Action::Execute(..)));
}
#[test]
fn dialog_can_cancel_and_busy_state_blocks_duplicate_signal() {
    let mut s = state();
    confirm(&mut s, false);
    s.busy = true;
    assert!(matches!(key(&mut s, KeyCode::Char('o')), Action::None));
    s.busy = false;
    key(&mut s, KeyCode::Esc);
    assert!(matches!(s.overlay, Overlay::None));
}
#[test]
fn minimum_dialog_keeps_confirmation_and_cancel_visible() {
    let mut s = state();
    confirm(&mut s, true);
    let text = render(&mut s, 60, 20);
    assert!(text.contains("Saisis le PID 1206"));
    assert!(text.contains("Échap annuler"));
    render(&mut s, 40, 10);
    for c in "1206".chars() {
        key(&mut s, KeyCode::Char(c));
    }
    assert!(matches!(key(&mut s, KeyCode::Enter), Action::None));
}
#[test]
fn refresh_retains_selected_endpoint_identity() {
    let mut s = state();
    s.move_selection(2);
    let id = s.selected().unwrap().id.clone();
    let mut next = s.snapshot.clone();
    next.endpoints.reverse();
    s.apply(next);
    assert_eq!(s.selected().unwrap().id, id);
}
#[test]
fn controls_ansi_bidi_and_wide_graphemes_cannot_break_layout() {
    assert_eq!(safe("a\x1b[31mb\x1b[0m\u{202e}c\x1b]0;bad\x07"), "abc");
    assert_eq!(fit("你好abc", 5), "你好…");
    assert_eq!(fit("abc", 0), "");
    let mut s = state();
    s.snapshot.endpoints[0].process.command = "node vite \x1b[2J\u{202e}evil".into();
    assert!(!render(&mut s, 80, 24).contains('\u{1b}'));
}
