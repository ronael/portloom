use super::system::local::timestamp;
use crate::{
    application::ports::{AppResult, PortScanner, ProcessControl, Signal},
    domain::model::{Endpoint, ProcessInfo, Project, Protocol, Snapshot},
};
use std::sync::Mutex;
pub struct DemoSystem {
    rows: Mutex<Vec<Endpoint>>,
}
impl Default for DemoSystem {
    fn default() -> Self {
        Self {
            rows: Mutex::new(fixtures()),
        }
    }
}
impl PortScanner for DemoSystem {
    fn scan(&self) -> AppResult<Snapshot> {
        Ok(Snapshot {
            endpoints: self.rows.lock().unwrap().clone(),
            captured_at: timestamp(),
        })
    }
}
impl ProcessControl for DemoSystem {
    fn inspect(&self, pid: u32) -> AppResult<Option<ProcessInfo>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.process.pid == pid)
            .map(|e| e.process.clone()))
    }
    fn signal(&self, pid: u32, _: Signal) -> AppResult<()> {
        self.rows.lock().unwrap().retain(|e| e.process.pid != pid);
        Ok(())
    }
}
pub fn fixtures() -> Vec<Endpoint> {
    use Protocol::{Tcp, Udp};
    let rows = [
        (
            3000,
            "node",
            8421,
            Tcp,
            "127.0.0.1",
            "node node_modules/vite/bin/vite.js --port 3000",
            2.4,
            184320,
        ),
        (
            4321,
            "bun",
            8530,
            Tcp,
            "::1",
            "bun run astro dev",
            0.8,
            94208,
        ),
        (
            5432,
            "postgres",
            1206,
            Tcp,
            "127.0.0.1",
            "postgres -D /opt/homebrew/var/postgresql",
            0.2,
            43008,
        ),
        (
            6379,
            "redis",
            1198,
            Tcp,
            "127.0.0.1",
            "redis-server 127.0.0.1:6379",
            0.1,
            12288,
        ),
        (
            8080,
            "node",
            8421,
            Tcp,
            "*",
            "node node_modules/vite/bin/vite.js --port 3000",
            2.4,
            184320,
        ),
        (
            5353,
            "Google Chrome Helper",
            3240,
            Udp,
            "*",
            "Google Chrome Helper --utility",
            0.0,
            8192,
        ),
        (
            5000,
            "ControlCenter",
            700,
            Tcp,
            "*",
            "ControlCenter",
            0.1,
            28672,
        ),
        (
            7000,
            "ControlCenter",
            700,
            Tcp,
            "*",
            "ControlCenter",
            0.1,
            28672,
        ),
        (
            55432,
            "com.docker.backend",
            60182,
            Tcp,
            "*",
            "com.docker.backend",
            1.2,
            327680,
        ),
        (
            4567,
            "python3",
            5400,
            Tcp,
            "127.0.0.1",
            "python3 worker.py",
            0.0,
            20480,
        ),
        (
            9229,
            "node",
            8532,
            Tcp,
            "127.0.0.1",
            "node --inspect worker.mjs",
            0.4,
            75776,
        ),
    ];
    rows.into_iter().map(|(port, name, pid, protocol, address, command, cpu, memory_kb)| {
        let project = match pid { 8421 | 8532 => Some("atelier-web"), 8530 => Some("portfolio"), _ => None }.map(|name| Project { name: name.into(), root: format!("/workspace/{name}"), branch: Some("main".into()) });
        Endpoint { id: format!("{}:{port}:{pid}", protocol.label()), port, protocol, addresses: vec![address.into()], process: ProcessInfo {
            pid, uid: 501, name: name.into(), command: command.into(), cpu, memory_kb, started_at: "Mon Oct 26 09:00:00 2026".into(),
            executable: Some(match pid { 700 => "/System/Library/CoreServices/ControlCenter.app/Contents/MacOS/ControlCenter", 3240 => "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", _ => name }.into()), cwd: project.as_ref().map(|p| p.root.clone()), project,
        }}
    }).collect()
}

impl crate::application::web::HttpProbe for DemoSystem {
    fn probe(
        &self,
        targets: &[crate::domain::web::HttpTarget],
    ) -> AppResult<crate::domain::web::HttpResponse> {
        Ok(crate::domain::web::HttpResponse {
            target: targets
                .first()
                .ok_or(crate::domain::issue::Issue::LoopbackRequired)?
                .clone(),
            status: 200,
            elapsed_ms: 0,
            simulated: true,
        })
    }
}
impl crate::application::web::BrowserOpener for DemoSystem {
    fn open(&self, _: &crate::domain::web::HttpTarget) -> AppResult<()> {
        Ok(())
    }
}
