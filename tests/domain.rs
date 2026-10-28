use portloom::{
    domain::{
        assessment::{Kind, assess},
        model::{Project, same_process},
        policy::{Actor, blocked},
    },
    infrastructure::demo::fixtures,
};
#[test]
fn sites_need_command_evidence_and_ports_do_not_prove_http() {
    let mut p = fixtures()[0].process.clone();
    assert_eq!(assess(&p).kind, Kind::Web);
    p.command = "node worker.js".into();
    assert_eq!(assess(&p).kind, Kind::Development);
    p.project = None;
    assert_eq!(assess(&p).kind, Kind::Unknown);
    assert!(assess(&p).needs_pid);
}
#[test]
fn system_ownership_and_paths_override_web_hints() {
    let mut p = fixtures()[0].process.clone();
    p.uid = 0;
    assert_eq!(assess(&p).kind, Kind::System);
    p.uid = 501;
    p.executable = Some("/usr/libexec/service".into());
    assert_eq!(assess(&p).kind, Kind::System);
    p.executable = None;
    p.name = "ControlCenter".into();
    assert_eq!(assess(&p).kind, Kind::System);
}
#[test]
fn apps_and_agents_win_over_project_runtime() {
    let mut p = fixtures()[0].process.clone();
    p.executable = Some("/Applications/Foo.app/Contents/MacOS/node".into());
    assert_eq!(assess(&p).kind, Kind::Application);
    p.executable = Some("/usr/local/bin/node".into());
    p.command = "node mcp-server.js".into();
    assert_eq!(assess(&p).kind, Kind::Application);
    p.command = "node worker.js".into();
    p.executable =
        Some("/project/node_modules/electron/Electron.app/Contents/MacOS/Electron".into());
    assert_eq!(assess(&p).kind, Kind::Development);
}
#[test]
fn python_framework_is_a_runtime() {
    let mut p = fixtures()[0].process.clone();
    p.name = "python3".into();
    p.command = "python3 -m http.server 4321".into();
    p.executable = Some("/Library/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python".into());
    assert_eq!(assess(&p).kind, Kind::Web);
}
#[test]
fn shared_services_require_pid() {
    for name in ["postgres", "redis-server", "com.docker.backend", "nginx"] {
        let mut p = fixtures()[0].process.clone();
        p.name = name.into();
        p.executable = Some(name.into());
        let a = assess(&p);
        assert!(matches!(a.kind, Kind::Service | Kind::Database));
        assert!(a.needs_pid);
    }
}
#[test]
fn stop_policy_blocks_system_other_user_self_parent_and_unknown_identity() {
    let actor = Actor {
        uid: 501,
        protected_pids: vec![8421, 2],
    };
    let original = fixtures()[0].process.clone();
    assert!(blocked(&original, &actor).is_some());
    let mut p = original.clone();
    p.pid = 42;
    assert!(blocked(&p, &actor).is_none());
    p.uid = 502;
    assert!(blocked(&p, &actor).is_some());
    p.uid = 501;
    p.started_at.clear();
    assert!(blocked(&p, &actor).is_some());
    p = original;
    p.pid = 0;
    assert!(blocked(&p, &actor).is_some());
    p.pid = u32::MAX;
    assert!(blocked(&p, &actor).is_some());
}
#[test]
fn identity_never_matches_without_start_time() {
    let mut p = fixtures()[0].process.clone();
    assert!(same_process(&p, &p));
    p.started_at.clear();
    assert!(!same_process(&p, &p));
}
#[test]
fn categorization_keeps_all_views_explainable() {
    let mut p = fixtures()[0].process.clone();
    p.project = Some(Project {
        name: "atelier".into(),
        root: "/workspace/atelier".into(),
        branch: None,
    });
    assert!(assess(&p).title.contains("atelier"));
    assert!(Kind::Web.in_view(1));
    assert!(!Kind::System.in_view(1));
    assert!(Kind::Database.in_view(2));
    assert!(Kind::Unknown.in_view(5));
}
