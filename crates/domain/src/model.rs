#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    pub name: String,
    pub root: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub uid: u32,
    pub name: String,
    pub command: String,
    pub started_at: String,
    pub cpu: f64,
    pub memory_kb: u64,
    pub executable: Option<String>,
    pub cwd: Option<String>,
    pub project: Option<Project>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    Tcp,
    Udp,
}
impl Protocol {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub id: String,
    pub port: u16,
    pub protocol: Protocol,
    pub addresses: Vec<String>,
    pub process: ProcessInfo,
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub endpoints: Vec<Endpoint>,
    pub captured_at: u64,
}
pub fn same_process(a: &ProcessInfo, b: &ProcessInfo) -> bool {
    a.pid == b.pid && a.uid == b.uid && !a.started_at.is_empty() && a.started_at == b.started_at
}
