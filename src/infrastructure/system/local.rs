use super::{command::command, context::ProjectFinder, parsers};
use crate::{
    application::ports::{AppResult, Clock, PortScanner, ProcessControl, Signal},
    domain::{
        model::{ProcessInfo, Snapshot},
        policy::Actor,
    },
};
use std::{
    collections::BTreeSet,
    sync::{Arc, LazyLock},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Default)]
pub struct LocalScanner {
    projects: ProjectFinder,
}
pub struct LocalControl;
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> u64 {
        static START: LazyLock<Instant> = LazyLock::new(Instant::now);
        START.elapsed().as_millis() as u64
    }
    fn sleep(&self, millis: u64) {
        thread::sleep(Duration::from_millis(millis));
    }
}
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub fn actor() -> Actor {
    Actor {
        uid: nix::unistd::getuid().as_raw(),
        protected_pids: vec![std::process::id(), nix::unistd::getppid().as_raw() as u32],
    }
}
fn process_list(pids: &str) -> AppResult<std::collections::BTreeMap<u32, ProcessInfo>> {
    Ok(parsers::ps(&command(
        "ps",
        &[
            "-ww",
            "-p",
            pids,
            "-o",
            "pid=,uid=,lstart=,pcpu=,rss=,stat=,args=",
        ],
    )?))
}
fn executable_list(pids: &str) -> AppResult<std::collections::BTreeMap<u32, String>> {
    let mut paths = parsers::executables(&command("ps", &["-ww", "-p", pids, "-o", "pid=,comm="])?);
    if cfg!(target_os = "linux") {
        for (pid, path) in &mut paths {
            if let Ok(exe) = std::fs::read_link(format!("/proc/{pid}/exe")) {
                *path = exe.to_string_lossy().into_owned();
            }
        }
    }
    Ok(paths)
}
impl PortScanner for LocalScanner {
    fn scan(&self) -> AppResult<Snapshot> {
        let (tcp, udp) = thread::scope(|scope| {
            let tcp =
                scope.spawn(|| command("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN", "-FpcuftPn"]));
            let udp = command("lsof", &["-nP", "-iUDP", "-FpcuftPn"]);
            (
                tcp.join()
                    .unwrap_or_else(|_| Err("Scan TCP interrompu.".into())),
                udp,
            )
        });
        let mut sockets = parsers::lsof(&tcp?);
        sockets.extend(parsers::lsof(&udp?));
        let pids = sockets
            .iter()
            .map(|s| s.pid)
            .collect::<BTreeSet<_>>()
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        if pids.is_empty() {
            return Ok(Snapshot {
                endpoints: vec![],
                captured_at: timestamp(),
            });
        }
        let (processes, executables, cwd) = thread::scope(|scope| {
            let p = scope.spawn(|| process_list(&pids));
            let e = scope.spawn(|| executable_list(&pids));
            let cwd = command("lsof", &["-a", "-p", &pids, "-d", "cwd", "-Fpn"])
                .map(|text| parsers::directories(&text))
                .unwrap_or_default();
            (
                p.join()
                    .unwrap_or_else(|_| Err("Lecture ps interrompue.".into())),
                e.join()
                    .unwrap_or_else(|_| Err("Lecture des exécutables interrompue.".into())),
                cwd,
            )
        });
        let mut processes = processes?;
        let mut executables = executables?;
        for (pid, process) in &mut processes {
            process.executable = executables.remove(pid);
            process.cwd = cwd.get(pid).cloned();
            process.project = process
                .cwd
                .as_deref()
                .and_then(|cwd| self.projects.find(cwd));
        }
        Ok(Snapshot {
            endpoints: parsers::join(sockets, processes),
            captured_at: timestamp(),
        })
    }
}
impl ProcessControl for LocalControl {
    fn inspect(&self, pid: u32) -> AppResult<Option<ProcessInfo>> {
        let pids = pid.to_string();
        let Some(mut p) = process_list(&pids)?.remove(&pid) else {
            return Ok(None);
        };
        p.executable = executable_list(&pids)?.remove(&pid);
        Ok(Some(p))
    }
    fn signal(&self, pid: u32, signal: Signal) -> AppResult<()> {
        if pid <= 1 || pid > i32::MAX as u32 {
            return Err("PID invalide : aucun signal envoyé.".into());
        }
        let sig = match signal {
            Signal::Terminate => nix::sys::signal::Signal::SIGTERM,
            Signal::Kill => nix::sys::signal::Signal::SIGKILL,
        };
        match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid as i32), sig) {
            Ok(()) | Err(nix::errno::Errno::ESRCH) => Ok(()),
            Err(e) => Err(format!("Signal refusé pour le PID {pid} : {e}.")),
        }
    }
}
pub fn clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}
