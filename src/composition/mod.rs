mod adapters;
use crate::{
    application::preferences::{self, Language, PreferencesStore},
    infrastructure::preferences::FilePreferences,
    presentation::{
        cli::{self, Cli, Command},
        console,
        design::safe,
        i18n::{Locale, T},
        runtime,
    },
};
use clap::FromArgMatches;
use std::{
    ffi::OsString,
    io::{self, IsTerminal},
    process::ExitCode,
    sync::Arc,
};

fn argument_language(args: &[OsString]) -> Option<Language> {
    for (index, arg) in args.iter().enumerate() {
        if arg == "--" {
            break;
        }
        if arg == "--lang" {
            return args
                .get(index + 1)
                .and_then(|s| s.to_str())
                .and_then(Language::from_code);
        }
        if let Some(value) = arg.to_str().and_then(|s| s.strip_prefix("--lang=")) {
            return Language::from_code(value);
        }
    }
    None
}
pub fn run() -> ExitCode {
    let args: Vec<_> = std::env::args_os().collect();
    let preferences: Arc<dyn PreferencesStore> = Arc::new(FilePreferences::from_environment());
    let saved = preferences.load();
    let explicit = argument_language(&args);
    let environment = std::env::var("PORTLOOM_LANG")
        .ok()
        .and_then(|s| Language::from_code(&s));
    let selected = saved.as_ref().ok().and_then(|p| p.language);
    let locale = Locale(preferences::resolve(explicit, environment, selected));
    let matches = cli::command(locale).get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    if let Some(Command::Completions { shell }) = cli.command {
        clap_complete::generate(
            shell,
            &mut cli::command(locale),
            "portloom",
            &mut io::stdout(),
        );
        return ExitCode::SUCCESS;
    }
    if let Err(error) = saved {
        eprintln!(
            "{}",
            safe(&locale.format(T::ConfigWarning, &[("error", &locale.error(&error))]))
        );
    }
    let first_language = if !cli.demo
        && !cli.json
        && explicit.is_none()
        && environment.is_none()
        && selected.is_none()
        && (cli.command.is_none() || matches!(cli.command, Some(Command::Watch { .. })))
        && io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && std::env::var_os("CI").is_none()
    {
        Some(
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .into_iter()
                .find_map(|key| std::env::var(key).ok().filter(|v| !v.is_empty()))
                .map(|value| Language::suggest(&value))
                .unwrap_or_default(),
        )
    } else {
        None
    };
    let (services, actor) = adapters::compose(cli.demo, preferences);
    let options = runtime::Options {
        actor,
        demo: cli.demo,
        interval: cli.interval,
        locale,
        first_language,
        port: None,
    };
    match console::execute(cli, services, options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(issue) => {
            eprintln!("Portloom: {}", safe(&locale.error(&issue)));
            ExitCode::FAILURE
        }
    }
}
