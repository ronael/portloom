use super::{AppResult, Issue, Locale, PortQuery, Services, T, safe, write};
use crate::application::stop::StopResult;
use std::io::{self, IsTerminal, Write};

pub fn run(
    services: &Services,
    query: PortQuery,
    force: bool,
    as_json: bool,
    locale: Locale,
) -> AppResult<()> {
    if as_json || !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(Issue::InteractiveRequired);
    }
    let target = query.target(&services.scanner.scan()?)?;
    let plan = services.stopper.prepare(&target, force)?;
    write(locale.text(if force {
        T::ForceTitle
    } else {
        T::ConfirmTitle
    }))?;
    write(&safe(&format!(
        "{} · PID {}",
        plan.assessment.title, plan.target.pid
    )))?;
    write(locale.assessment(&plan.assessment).impact)?;
    write(locale.text(if force { T::KillWarning } else { T::TermInfo }))?;
    let ports = plan
        .endpoints
        .iter()
        .map(|e| format!("{}:{}", e.protocol.label(), e.port))
        .collect::<Vec<_>>()
        .join(", ");
    write(&locale.format(T::AffectedPorts, &[("ports", &ports)]))?;
    write(&safe(
        &locale.format(T::CommandValue, &[("value", &plan.target.command)]),
    ))?;
    let answer = if plan.assessment.needs_pid {
        plan.target.pid.to_string()
    } else {
        locale.confirm_key().to_ascii_uppercase().to_string()
    };
    print!(
        "{}",
        locale.format(
            T::StopPrompt,
            &[("pid", &plan.target.pid.to_string()), ("answer", &answer)]
        )
    );
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    if !input.trim().eq_ignore_ascii_case(&answer) {
        return Err(Issue::Cancelled);
    }
    match services.stopper.execute(&plan, input.trim())? {
        StopResult::Stopped => write(locale.text(T::Stopped)),
        StopResult::Gone => write(locale.text(T::Gone)),
        StopResult::Running => Err(locale.text(T::StillRunning).into()),
    }
}
