use super::{AppResult, Locale, PortQuery, Services, json};
use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
pub fn run(
    services: &Services,
    port: u16,
    count: Option<u32>,
    interval: u64,
    locale: Locale,
) -> AppResult<()> {
    let quit = Arc::new(AtomicBool::new(false));
    let mut registrations = Vec::new();
    for signal in [
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ] {
        registrations
            .push(signal_hook::flag::register(signal, quit.clone()).map_err(|e| e.to_string())?);
    }
    let result = (|| -> AppResult<()> {
        let query = PortQuery {
            port: Some(port),
            ..Default::default()
        };
        let mut emitted = 0;
        while !quit.load(Ordering::Relaxed) {
            let snapshot = query.filter(services.scanner.scan()?);
            let value = json::localized_snapshot(&snapshot, locale);
            match writeln!(io::stdout().lock(), "{value}") {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::BrokenPipe => break,
                Err(e) => return Err(e.to_string().into()),
            }
            emitted += 1;
            if count.is_some_and(|limit| emitted >= limit) {
                break;
            }
            let deadline = Instant::now() + Duration::from_millis(interval);
            while Instant::now() < deadline && !quit.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(25));
            }
        }
        Ok(())
    })();
    for registration in registrations {
        signal_hook::low_level::unregister(registration);
    }
    result
}
