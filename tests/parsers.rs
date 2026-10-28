use portloom::infrastructure::system::parsers;
#[test]
fn lsof_deduplicates_families_and_uses_connected_udp_local_port() {
    let sockets = parsers::lsof(
        "p42\ncnode\nu501\nf4\nPTCP\nn127.0.0.1:3000\nf5\nPTCP\nn[::1]:3000\nf6\nPUDP\nn127.0.0.1:4567->127.0.0.1:9\n",
    );
    let p = parsers::ps("42 501 Mon Oct 26 09:00:00 2026 2.4 1234 S node worker.js\n");
    let joined = parsers::join(sockets, p);
    assert_eq!(joined.len(), 2);
    assert_eq!(joined[0].addresses, ["127.0.0.1", "::1"]);
    assert_eq!(joined[1].port, 4567);
}
#[test]
fn ignores_invalid_fields_disappeared_processes_and_ownership_races() {
    let sockets = parsers::lsof(
        "p42\nu501\nf3\nn*:9000\nPTCP\nn*:0\nn*:70000\nn*:12\np0\nu501\nf3\nPTCP\nn*:13\n",
    );
    assert_eq!(sockets.len(), 1);
    assert!(
        parsers::join(
            sockets,
            parsers::ps("42 502 Mon Oct 26 09:00:00 2026 0.0 1234 S node\n")
        )
        .is_empty()
    );
}
#[test]
fn ps_preserves_full_commands_and_normalizes_start_time() {
    let p = parsers::ps("  42 501 Mon Oct  6 09:00:00 2026 1.3 234 S+ /Applications/Some App.app/Contents/bin --arg\n").remove(&42).unwrap();
    assert_eq!(p.started_at, "Mon Oct 6 09:00:00 2026");
    assert!(p.command.ends_with("bin --arg"));
    assert_eq!(
        parsers::executables("42 /Applications/Some App.app/Contents/bin\n")[&42],
        "/Applications/Some App.app/Contents/bin"
    );
}
#[test]
fn zombies_and_unparseable_rows_are_not_actionable() {
    assert!(parsers::ps("42 501 Mon Oct 26 09:00:00 2026 0.0 0 Z <defunct>\ninvalid\n").is_empty());
}
#[test]
fn cwd_keeps_spaces_but_rejects_relative_paths() {
    let dirs = parsers::directories("p42\nfcwd\nn/workspace/my app\np43\nnrelative\n");
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[&42], "/workspace/my app");
}
