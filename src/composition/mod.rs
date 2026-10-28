use crate::{
    application::{
        ports::{PortScanner, ProcessControl},
        stop::StopProcess,
    },
    domain::policy::Actor,
    infrastructure::{
        demo::DemoSystem,
        system::local::{self, LocalControl, LocalScanner},
    },
    presentation::{
        cli::{Cli, Command},
        json, runtime,
    },
};
use clap::{CommandFactory, Parser};
use std::{
    io::{self, Write},
    process::ExitCode,
    sync::Arc,
};
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    if let Some(Command::Completions { shell }) = cli.command {
        clap_complete::generate(shell, &mut Cli::command(), "portloom", &mut io::stdout());
        return ExitCode::SUCCESS;
    }
    match execute(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Portloom : {e}");
            ExitCode::FAILURE
        }
    }
}
fn execute(cli: Cli) -> Result<(), String> {
    let (scanner, control, actor): (Arc<dyn PortScanner>, Arc<dyn ProcessControl>, Actor) =
        if cli.demo {
            let demo = Arc::new(DemoSystem::default());
            (
                demo.clone(),
                demo,
                Actor {
                    uid: 501,
                    protected_pids: vec![],
                },
            )
        } else {
            (
                Arc::new(LocalScanner::default()),
                Arc::new(LocalControl),
                local::actor(),
            )
        };
    if cli.json {
        let data = serde_json::to_string_pretty(&json::snapshot(&scanner.scan()?))
            .map_err(|e| e.to_string())?;
        return match writeln!(io::stdout().lock(), "{data}") {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e.to_string()),
        };
    }
    let stopper = Arc::new(StopProcess::new(
        scanner.clone(),
        control,
        actor.clone(),
        local::clock(),
    ));
    runtime::run(scanner, stopper, actor, cli.demo, cli.interval)
}
