mod stop;
mod watch;
use super::{
    cli::{Cli, Command},
    design::safe,
    i18n::{Locale, T},
    json, runtime,
    tasks::Services,
};
use crate::{
    application::{ports::AppResult, query::PortQuery},
    domain::{assessment::assess, issue::Issue, model::Snapshot},
};
use std::io::{self, Write};

pub fn write(text: &str) -> AppResult<()> {
    match writeln!(io::stdout().lock(), "{text}") {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e.to_string().into()),
    }
}
fn output(
    snapshot: &Snapshot,
    locale: Locale,
    as_json: bool,
    details: bool,
    all: &Snapshot,
) -> AppResult<()> {
    if as_json {
        return write(
            &serde_json::to_string_pretty(&json::localized_snapshot(snapshot, locale))
                .map_err(|e| e.to_string())?,
        );
    }
    if snapshot.endpoints.is_empty() {
        return write(locale.text(T::NoRows));
    }
    if !details {
        write(&format!(
            "PORT   NET  PID      {:8}  {}",
            locale.text(T::Usage),
            locale.text(T::ProjectService)
        ))?;
    }
    for endpoint in &snapshot.endpoints {
        let p = &endpoint.process;
        let assessment = assess(p);
        let text = locale.assessment(&assessment);
        write(&safe(&format!(
            "{:<6} {:3}  {:<8} {:8}  {}",
            endpoint.port,
            endpoint.protocol.label(),
            p.pid,
            text.badge,
            assessment.title
        )))?;
        if details {
            for line in [text.label, text.reason.as_str(), text.impact] {
                write(&safe(&format!("  {line}")))?;
            }
            for (label, value) in [
                (
                    T::Directory,
                    p.cwd.as_deref().unwrap_or(locale.text(T::Unavailable)),
                ),
                (T::Command, p.command.as_str()),
                (
                    T::Executable,
                    p.executable
                        .as_deref()
                        .unwrap_or(locale.text(T::Unavailable)),
                ),
                (T::Started, p.started_at.as_str()),
            ] {
                write(&safe(&format!("  {}: {value}", locale.text(label))))?;
            }
            if let Some(branch) = p.project.as_ref().and_then(|p| p.branch.as_deref()) {
                write(&safe(&format!("  {}: {branch}", locale.text(T::Branch))))?;
            }
            let ports = all
                .endpoints
                .iter()
                .filter(|e| e.process.pid == p.pid)
                .map(|e| format!("{}:{}", e.protocol.label(), e.port))
                .collect::<Vec<_>>()
                .join(", ");
            write(&locale.format(T::PidPorts, &[("ports", &ports)]))?;
            write("")?;
        }
    }
    Ok(())
}
pub fn execute(cli: Cli, services: Services, mut options: runtime::Options) -> AppResult<()> {
    let locale = options.locale;
    match cli.command {
        Some(Command::Language { language }) => {
            services.preferences.save_language(language)?;
            if cli.json {
                write(&serde_json::json!({"language":language.code(),"saved":true}).to_string())
            } else {
                write(&Locale(language).format(T::SavedCode, &[("language", language.code())]))
            }
        }
        Some(Command::Completions { .. }) => unreachable!("handled before runtime composition"),
        Some(Command::Stop { target, force }) => {
            stop::run(&services, target.query(), force, cli.json, locale)
        }
        Some(Command::Probe(target)) => web(&services, target.query(), false, cli.json, locale),
        Some(Command::Open(target)) => web(&services, target.query(), true, cli.json, locale),
        Some(Command::Inspect(target)) => {
            let all = services.scanner.scan()?;
            let snapshot = target.query().filter(all.clone());
            if snapshot.endpoints.is_empty() {
                return Err(Issue::PortNotFound(target.port));
            }
            output(&snapshot, locale, cli.json, true, &all)
        }
        Some(Command::List { projects }) => {
            let all = services.scanner.scan()?;
            let snapshot = PortQuery {
                projects_only: projects,
                ..Default::default()
            }
            .filter(all.clone());
            output(&snapshot, locale, cli.json, false, &all)
        }
        Some(Command::Watch { port, count }) if cli.json => {
            watch::run(&services, port, count, cli.interval, locale)
        }
        Some(Command::Watch { port, .. }) => {
            options.port = Some(port);
            runtime::run(services, options)
        }
        None if cli.json => {
            let snapshot = services.scanner.scan()?;
            output(&snapshot, locale, true, false, &snapshot)
        }
        None => runtime::run(services, options),
    }
}
fn web(
    services: &Services,
    query: PortQuery,
    open: bool,
    as_json: bool,
    locale: Locale,
) -> AppResult<()> {
    let target = query.target(&services.scanner.scan()?)?;
    let report = services.web.check(&target, open)?;
    let response = report.response;
    if as_json {
        write(&serde_json::json!({"port":target.port,"pid":target.process.pid,"url":response.target.url(),"status":response.status,"elapsedMs":response.elapsed_ms,"opened":open,"simulated":response.simulated}).to_string())
    } else {
        let key = if open {
            if response.simulated {
                T::DemoOpened
            } else {
                T::Opened
            }
        } else {
            T::HttpResult
        };
        write(&locale.format(
            key,
            &[
                ("url", &response.target.url()),
                ("status", &response.status.to_string()),
                ("ms", &response.elapsed_ms.to_string()),
            ],
        ))
    }
}
