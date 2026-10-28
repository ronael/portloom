use super::support::Directory;
use portloom::{
    application::preferences::{Language, PreferencesStore, resolve},
    infrastructure::preferences::FilePreferences,
    presentation::i18n::{Locale, T},
};
use std::collections::BTreeSet;

#[test]
fn english_default_and_noninteractive_commands_never_prompt_or_write_preferences() {
    let dir = Directory::new();
    let help = dir.cli(&["--help"], None);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Your ports, under control"));
    let output = dir.cli(&["--demo", "inspect", "3000", "--json"], None);
    assert!(output.status.success(), "{:?}", output);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["endpoints"][0]["assessment"]["badge"], "WEB/API");
    assert!(!dir.0.join("settings.json").exists());
    assert!(
        !dir.cli(&["--lang", "de", "--demo", "--json"], None)
            .status
            .success()
    );
}
#[test]
fn saved_language_environment_and_explicit_flag_have_documented_precedence() {
    let dir = Directory::new();
    assert!(dir.cli(&["language", "fr"], None).status.success());
    assert!(
        String::from_utf8_lossy(&dir.cli(&["--help"], None).stdout)
            .contains("Tes ports, sous contrôle")
    );
    assert!(
        String::from_utf8_lossy(&dir.cli(&["--help"], Some("en")).stdout).contains("Your ports")
    );
    assert!(
        String::from_utf8_lossy(
            &dir.cli(&["--lang", "fr", "inspect", "--help"], Some("en"))
                .stdout
        )
        .contains("Inspecter")
    );
    let en = dir.cli(&["--demo", "--lang", "en", "--json"], None);
    let fr = dir.cli(&["--demo", "--lang", "fr", "--json"], None);
    let en: serde_json::Value = serde_json::from_slice(&en.stdout).unwrap();
    let fr: serde_json::Value = serde_json::from_slice(&fr.stdout).unwrap();
    assert_eq!(
        en["endpoints"][0]["assessment"]["kind"],
        fr["endpoints"][0]["assessment"]["kind"]
    );
    assert_eq!(en["endpoints"][0]["process"], fr["endpoints"][0]["process"]);
    assert_ne!(
        en["endpoints"][0]["assessment"]["badge"],
        fr["endpoints"][0]["assessment"]["badge"]
    );
    assert_eq!(
        FilePreferences::new(dir.0.clone()).load().unwrap().language,
        Some(Language::French)
    );
    assert_eq!(resolve(None, None, None), Language::English);
    assert_eq!(Language::suggest("fr_FR.UTF-8"), Language::French);
    assert_eq!(Language::suggest("fr-CA"), Language::French);
    assert_eq!(Language::suggest("de_DE.UTF-8"), Language::English);
}
#[test]
fn preference_writes_are_private_preserve_other_fields_and_recover_invalid_data() {
    let dir = Directory::new();
    let path = dir.0.join("settings.json");
    std::fs::write(&path, r#"{"theme":"dark","language":"en"}"#).unwrap();
    let store = FilePreferences::new(dir.0.clone());
    store.save_language(Language::French).unwrap();
    let data: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(data["theme"], "dark");
    assert_eq!(data["language"], "fr");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    std::fs::write(&path, "invalid JSON").unwrap();
    assert!(store.load().is_err());
    store.save_language(Language::English).unwrap();
    assert_eq!(store.load().unwrap().language, Some(Language::English));
    std::fs::write(&path, " ".repeat(16_385)).unwrap();
    assert!(store.load().is_err());
    assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
}
#[test]
fn translations_keep_interpolation_fields_in_both_languages() {
    fn fields(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .filter_map(|s| s.split_once('}').map(|v| v.0))
            .collect()
    }
    for &key in T::ALL {
        let en = Locale::EN.text(key);
        let fr = Locale::FR.text(key);
        assert!(!en.is_empty() && !fr.is_empty(), "{key:?}");
        assert_eq!(fields(en), fields(fr), "{key:?}");
    }
}
#[test]
fn direct_commands_filter_ports_and_refuse_piped_stops() {
    let dir = Directory::new();
    let output = dir.cli(&["list", "--projects", "--demo", "--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["endpoints"].as_array().unwrap().len(), 4);
    for args in [
        &["stop", "3000", "--demo"][..],
        &["stop", "5432", "--demo", "--force"],
        &["inspect", "65000", "--demo"],
        &["watch", "3000", "--demo", "--count", "2"],
    ] {
        assert!(!dir.cli(args, None).status.success(), "{args:?}");
    }
    let output = dir.cli(
        &[
            "watch",
            "3000",
            "--demo",
            "--json",
            "--count",
            "2",
            "--interval",
            "500",
        ],
        None,
    );
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    let snapshots = text
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(snapshots.len(), 2);
    assert!(snapshots.iter().all(
        |s| s["endpoints"].as_array().unwrap().len() == 1 && s["endpoints"][0]["port"] == 3000
    ));
    let demo = dir.cli(&["open", "3000", "--demo", "--json"], None);
    let value: serde_json::Value = serde_json::from_slice(&demo.stdout).unwrap();
    assert_eq!(value["simulated"], true);
    assert_eq!(value["opened"], true);
    assert!(!dir.0.join("settings.json").exists());
}
