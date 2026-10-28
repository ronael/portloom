use portloom::{
    application::{
        ports::{AppResult, Clock, PortScanner, ProcessControl, Signal},
        stop::{StopProcess, StopResult},
    },
    domain::{
        model::{Endpoint, ProcessInfo, Snapshot},
        policy::Actor,
    },
    infrastructure::demo::fixtures,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
struct Fake {
    rows: Mutex<Vec<Endpoint>>,
    live: Mutex<Option<ProcessInfo>>,
    signals: Mutex<Vec<Signal>>,
    stay: bool,
}
impl PortScanner for Fake {
    fn scan(&self) -> AppResult<Snapshot> {
        Ok(Snapshot {
            endpoints: self.rows.lock().unwrap().clone(),
            captured_at: 0,
        })
    }
}
impl ProcessControl for Fake {
    fn inspect(&self, _: u32) -> AppResult<Option<ProcessInfo>> {
        Ok(self.live.lock().unwrap().clone())
    }
    fn signal(&self, _: u32, signal: Signal) -> AppResult<()> {
        self.signals.lock().unwrap().push(signal);
        if !self.stay {
            *self.live.lock().unwrap() = None;
        }
        Ok(())
    }
}
#[derive(Default)]
struct Time(AtomicU64);
impl Clock for Time {
    fn now(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
    fn sleep(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::Relaxed);
    }
}
fn setup(sensitive: bool, stay: bool) -> (Arc<Fake>, StopProcess, Endpoint) {
    let e = fixtures()[if sensitive { 2 } else { 0 }].clone();
    let fake = Arc::new(Fake {
        rows: Mutex::new(vec![e.clone()]),
        live: Mutex::new(Some(e.process.clone())),
        signals: Mutex::new(vec![]),
        stay,
    });
    let stop = StopProcess::new(
        fake.clone(),
        fake.clone(),
        Actor {
            uid: 501,
            protected_pids: vec![],
        },
        Arc::new(Time::default()),
    );
    (fake, stop, e)
}
#[test]
fn preparation_never_signals_and_normal_stop_uses_term() {
    let (fake, stop, e) = setup(false, false);
    let plan = stop.prepare(&e, false).unwrap();
    assert!(fake.signals.lock().unwrap().is_empty());
    assert_eq!(stop.execute(&plan, "").unwrap(), StopResult::Stopped);
    assert_eq!(*fake.signals.lock().unwrap(), vec![Signal::Terminate]);
}
#[test]
fn sensitive_stop_requires_exact_pid_even_outside_ui() {
    let (fake, stop, e) = setup(true, false);
    let plan = stop.prepare(&e, false).unwrap();
    for ack in ["", "o", "01206", "1206 "] {
        assert!(stop.execute(&plan, ack).is_err());
    }
    assert!(fake.signals.lock().unwrap().is_empty());
    assert_eq!(stop.execute(&plan, "1206").unwrap(), StopResult::Stopped);
}
#[test]
fn explicit_force_uses_kill_and_never_escalates_automatically() {
    let (fake, stop, e) = setup(false, true);
    let plan = stop.prepare(&e, false).unwrap();
    assert_eq!(stop.execute(&plan, "").unwrap(), StopResult::Running);
    assert_eq!(*fake.signals.lock().unwrap(), vec![Signal::Terminate]);
    let (fake, stop, e) = setup(false, false);
    let plan = stop.prepare(&e, true).unwrap();
    stop.execute(&plan, "").unwrap();
    assert_eq!(*fake.signals.lock().unwrap(), vec![Signal::Kill]);
}
#[test]
fn reused_pid_is_blocked_at_scan_and_final_inspection() {
    for at_scan in [true, false] {
        let (fake, stop, e) = setup(false, false);
        let plan = stop.prepare(&e, false).unwrap();
        if at_scan {
            fake.rows.lock().unwrap()[0].process.started_at = "changed".into();
        } else {
            fake.live.lock().unwrap().as_mut().unwrap().started_at = "changed".into();
        }
        assert!(stop.execute(&plan, "").is_err());
        assert!(fake.signals.lock().unwrap().is_empty());
    }
}
#[test]
fn new_ports_require_a_new_confirmation() {
    let (fake, stop, e) = setup(false, false);
    let plan = stop.prepare(&e, false).unwrap();
    let mut added = e;
    added.id = "TCP:8888:8421".into();
    added.port = 8888;
    fake.rows.lock().unwrap().push(added);
    assert!(stop.execute(&plan, "").unwrap_err().contains("autre port"));
    assert!(fake.signals.lock().unwrap().is_empty());
}
#[test]
fn classification_change_requires_new_confirmation() {
    let (fake, stop, e) = setup(false, false);
    let plan = stop.prepare(&e, false).unwrap();
    fake.rows.lock().unwrap()[0].process.command = "node worker.js".into();
    assert!(stop.execute(&plan, "").unwrap_err().contains("usage"));
    assert!(fake.signals.lock().unwrap().is_empty());
}
#[test]
fn final_inspection_cannot_bypass_system_protection() {
    let (fake, stop, e) = setup(false, false);
    let plan = stop.prepare(&e, false).unwrap();
    fake.live.lock().unwrap().as_mut().unwrap().executable = Some("/System/Library/service".into());
    assert!(stop.execute(&plan, "").is_err());
    assert!(fake.signals.lock().unwrap().is_empty());
}
#[test]
fn gone_ports_never_trigger_a_signal() {
    let (fake, stop, e) = setup(false, false);
    let plan = stop.prepare(&e, false).unwrap();
    fake.rows.lock().unwrap().clear();
    assert_eq!(stop.execute(&plan, "").unwrap(), StopResult::Gone);
    assert!(fake.signals.lock().unwrap().is_empty());
}
#[test]
fn prepare_includes_all_ports_owned_by_target() {
    let (fake, stop, e) = setup(false, false);
    fake.rows.lock().unwrap().push(fixtures()[4].clone());
    assert_eq!(stop.prepare(&e, false).unwrap().endpoints.len(), 2);
}
