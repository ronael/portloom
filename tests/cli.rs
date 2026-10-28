use std::process::Command;
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_portloom"))
        .env("PORTLOOM_LANG", "en")
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn help_version_and_completions_are_real_cli_commands() {
    let help = cli(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("--interval"));
    assert_eq!(
        String::from_utf8_lossy(&cli(&["--version"]).stdout).trim(),
        concat!("portloom ", env!("CARGO_PKG_VERSION"))
    );
    assert!(
        String::from_utf8_lossy(&cli(&["completions", "zsh"]).stdout).contains("#compdef portloom")
    );
}
#[test]
fn demo_json_preserves_snapshot_and_classification_contracts() {
    let output = cli(&["--demo", "--json"]);
    assert!(output.status.success());
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(data["capturedAt"].is_u64());
    let rows = data["endpoints"].as_array().unwrap();
    assert_eq!(rows.len(), 11);
    assert!(rows.iter().any(|e| e["assessment"]["kind"] == "system"));
    assert!(rows.iter().all(|e| e["process"]["startedAt"].is_string()));
}
#[test]
fn rejects_invalid_intervals_unknown_flags_and_non_tty_ui() {
    for args in [
        &["--interval", "499"][..],
        &["--interval", "60001"],
        &["--interval", "NaN"],
        &["--inexistent"],
        &["--demo"],
    ] {
        assert!(!cli(args).status.success(), "{args:?}");
    }
}
