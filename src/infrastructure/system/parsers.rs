use crate::domain::model::{Endpoint, ProcessInfo, Protocol};
use regex::Regex;
use std::{collections::BTreeMap, sync::LazyLock};
#[derive(Debug)]
pub struct Socket {
    pub pid: u32,
    pub uid: u32,
    pub name: String,
    pub protocol: Protocol,
    pub port: u16,
    pub address: String,
}
pub fn lsof(text: &str) -> Vec<Socket> {
    let mut rows = Vec::new();
    let mut pid = 0;
    let mut uid = u32::MAX;
    let mut name = String::new();
    let mut protocol = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_at_checked(1) else {
            continue;
        };
        match key {
            "p" => {
                pid = value.parse().unwrap_or(0);
                uid = u32::MAX;
                name.clear();
                protocol = None;
            }
            "u" => uid = value.parse().unwrap_or(u32::MAX),
            "c" => name = value.into(),
            "f" => protocol = None,
            "P" => {
                protocol = match value {
                    "TCP" => Some(Protocol::Tcp),
                    "UDP" => Some(Protocol::Udp),
                    _ => None,
                }
            }
            "n" => {
                let local = value.split("->").next().unwrap_or("");
                if let (Some(protocol), Some((host, port))) = (protocol, local.rsplit_once(':'))
                    && let Ok(port) = port.parse::<u16>()
                    && pid > 0
                    && port > 0
                {
                    rows.push(Socket {
                        pid,
                        uid,
                        name: name.clone(),
                        protocol,
                        port,
                        address: host.trim_start_matches('[').trim_end_matches(']').into(),
                    });
                }
            }
            _ => {}
        }
    }
    rows
}
static PS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(\d+)\s+(\d+)\s+(\S+\s+\S+\s+\d+\s+\S+\s+\d+)\s+([\d.]+)\s+(\d+)\s+(\S+)\s+(.+)$",
    )
    .unwrap()
});
pub fn ps(text: &str) -> BTreeMap<u32, ProcessInfo> {
    text.lines()
        .filter_map(|line| {
            let c = PS.captures(line)?;
            if c[6].starts_with(['Z', 'X']) {
                return None;
            }
            let pid = c[1].parse().ok()?;
            Some((
                pid,
                ProcessInfo {
                    pid,
                    uid: c[2].parse().ok()?,
                    name: c[7].split_whitespace().next()?.rsplit('/').next()?.into(),
                    command: c[7].into(),
                    started_at: c[3].split_whitespace().collect::<Vec<_>>().join(" "),
                    cpu: c[4].parse().ok()?,
                    memory_kb: c[5].parse().ok()?,
                    executable: None,
                    cwd: None,
                    project: None,
                },
            ))
        })
        .collect()
}
pub fn executables(text: &str) -> BTreeMap<u32, String> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (pid, name) = line.split_once(char::is_whitespace)?;
            Some((pid.parse().ok()?, name.trim_start().to_string()))
        })
        .collect()
}
pub fn directories(text: &str) -> BTreeMap<u32, String> {
    let mut rows = BTreeMap::new();
    let mut pid = 0;
    for line in text.lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.parse().unwrap_or(0);
        }
        if let Some(n) = line.strip_prefix('n')
            && pid > 0
            && n.starts_with('/')
        {
            rows.insert(pid, n.into());
        }
    }
    rows
}
pub fn join(sockets: Vec<Socket>, processes: BTreeMap<u32, ProcessInfo>) -> Vec<Endpoint> {
    let mut grouped: BTreeMap<String, Endpoint> = BTreeMap::new();
    for s in sockets {
        let Some(p) = processes.get(&s.pid).filter(|p| p.uid == s.uid) else {
            continue;
        };
        let id = format!("{}:{}:{}", s.protocol.label(), s.port, s.pid);
        let e = grouped.entry(id.clone()).or_insert_with(|| {
            let mut p = p.clone();
            if !s.name.is_empty() {
                p.name = s.name.clone();
            }
            Endpoint {
                id,
                port: s.port,
                protocol: s.protocol,
                addresses: Vec::new(),
                process: p,
            }
        });
        if !e.addresses.contains(&s.address) {
            e.addresses.push(s.address);
        }
    }
    let mut rows: Vec<_> = grouped.into_values().collect();
    rows.sort_by_key(|e| (e.port, e.protocol, e.process.pid));
    rows
}
