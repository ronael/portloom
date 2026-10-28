use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portloom::{
    application::{
        ports::PortScanner,
        query::{PortQuery, project_groups},
    },
    domain::{issue::Issue, policy::Actor},
    infrastructure::demo::DemoSystem,
    presentation::{
        grouping::VisibleRow,
        i18n::Locale,
        state::{Action, Overlay, State},
    },
};

fn key(state: &mut State, key: KeyCode) -> Action {
    state.key(KeyEvent::new(key, KeyModifiers::NONE))
}
fn state() -> State {
    let mut state = State::new(
        Actor {
            uid: 501,
            protected_pids: vec![],
        },
        true,
    );
    state.apply(DemoSystem::default().scan().unwrap());
    state
}
#[test]
fn projects_group_by_root_and_leave_shared_services_outside() {
    let mut snapshot = DemoSystem::default().scan().unwrap();
    snapshot.endpoints[1].process.project.as_mut().unwrap().name = "atelier-web".into();
    snapshot.endpoints[2].process.project = snapshot.endpoints[0].process.project.clone();
    let groups = project_groups(
        &snapshot,
        &(0..snapshot.endpoints.len()).collect::<Vec<_>>(),
    );
    assert_eq!(groups.len(), 2);
    assert_eq!(groups.iter().map(|g| g.endpoints.len()).sum::<usize>(), 4);
    assert_ne!(groups[0].root, groups[1].root);
    assert!(!groups.iter().any(|g| g.endpoints.contains(&2)));
}
#[test]
fn collapsed_headers_cannot_stop_or_probe_hidden_processes_and_search_expands_matches() {
    let mut state = state();
    key(&mut state, KeyCode::Char('g'));
    state.table.select(Some(0));
    assert!(matches!(state.selected_row(), Some(VisibleRow::Group(_))));
    let expanded = state.rows.len();
    assert!(matches!(key(&mut state, KeyCode::Char('k')), Action::None));
    assert!(matches!(key(&mut state, KeyCode::Char('w')), Action::None));
    key(&mut state, KeyCode::Enter);
    assert!(state.rows.len() < expanded);
    assert!(state.selected().is_none());
    state.query = "3000".into();
    state.rebuild();
    // Both Vite endpoints match: the command also contains --port 3000.
    assert_eq!(state.visible.len(), 2);
    assert_eq!(state.rows.len(), 3);
    state.move_selection(1);
    assert_eq!(state.selected().unwrap().port, 3000);
    let mut next = state.snapshot.clone();
    next.endpoints.reverse();
    state.apply(next);
    assert_eq!(state.selected().unwrap().port, 3000);
    state.apply(Default::default());
    assert!(state.selected().is_none());
}
#[test]
fn query_requires_a_pid_for_ambiguous_owners() {
    let mut snapshot = DemoSystem::default().scan().unwrap();
    let mut other = snapshot.endpoints[0].clone();
    other.process.pid += 1;
    other.id = "other".into();
    snapshot.endpoints.push(other);
    let mut query = PortQuery {
        port: Some(3000),
        ..Default::default()
    };
    assert_eq!(
        query.target(&snapshot).unwrap_err(),
        Issue::AmbiguousPort(3000)
    );
    query.pid = Some(8421);
    assert_eq!(query.target(&snapshot).unwrap().process.pid, 8421);
}
#[test]
fn language_picker_has_keyboard_selection_and_english_confirm_uses_y() {
    let mut state = state();
    key(&mut state, KeyCode::Char('L'));
    key(&mut state, KeyCode::Char('2'));
    assert!(matches!(
        key(&mut state, KeyCode::Enter),
        Action::Language(portloom::application::preferences::Language::French)
    ));
    key(&mut state, KeyCode::Esc);
    assert!(matches!(state.overlay, Overlay::None));
    assert_eq!(state.locale, Locale::EN);
    let demo = std::sync::Arc::new(DemoSystem::default());
    let stop = portloom::application::stop::StopProcess::new(
        demo.clone(),
        demo,
        state.actor.clone(),
        portloom::infrastructure::system::local::clock(),
    );
    let plan = stop.prepare(state.selected().unwrap(), false).unwrap();
    state.overlay = Overlay::Confirm {
        plan: Box::new(plan),
        input: String::new(),
        scroll: 0,
    };
    assert!(matches!(key(&mut state, KeyCode::Char('o')), Action::None));
    assert!(matches!(key(&mut state, KeyCode::Enter), Action::None));
    assert!(matches!(
        key(&mut state, KeyCode::Char('y')),
        Action::Execute(..)
    ));
}
