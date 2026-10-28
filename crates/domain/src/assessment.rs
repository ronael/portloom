use super::model::ProcessInfo;
use regex::Regex;
use std::sync::LazyLock;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Web,
    Development,
    Unknown,
    Database,
    Service,
    Application,
    System,
}
impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Development => "development",
            Self::Unknown => "unknown",
            Self::Database => "database",
            Self::Service => "service",
            Self::Application => "application",
            Self::System => "system",
        }
    }
    pub fn in_view(self, view: u8) -> bool {
        match view {
            1 => matches!(self, Self::Web | Self::Development),
            2 => matches!(self, Self::Database | Self::Service),
            3 => self == Self::Application,
            4 => self == Self::System,
            5 => self == Self::Unknown,
            _ => true,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Evidence {
    Root,
    System(String),
    Database(String),
    Service(String),
    Application(String),
    Agent,
    Web(String),
    Project { runtime: String, root: String },
    Unknown,
}
#[derive(Clone, Debug)]
pub struct Assessment {
    pub kind: Kind,
    pub title: String,
    pub evidence: Evidence,
    pub needs_pid: bool,
}
impl Assessment {
    fn new(kind: Kind, title: String, evidence: Evidence) -> Self {
        Self {
            kind,
            title,
            evidence,
            needs_pid: !matches!(kind, Kind::Web | Kind::Development),
        }
    }
}
static WEB: LazyLock<Vec<(&str, Regex)>> = LazyLock::new(|| {
    [
        ("Vite", r"(?i)(?:^|[ /])vite(?:\.js)?(?:\s|$)"),
        (
            "Next.js",
            r"(?i)(?:^|[ /])next(?:-server|(?:\.js)?\s+(?:dev|start))(?:\s|$)",
        ),
        (
            "Astro",
            r"(?i)(?:^|[ /])astro(?:\.mjs)?\s+(?:dev|preview)(?:\s|$)",
        ),
        (
            "Nuxt",
            r"(?i)(?:^|[ /])nuxt(?:\.mjs)?\s+(?:dev|start|preview)(?:\s|$)",
        ),
        ("Python HTTP", r"(?i)\bpython\S*\s+-m\s+http\.server\b"),
        ("Uvicorn", r"(?i)(?:^|[ /])uvicorn(?:\s|$)"),
        ("Django", r"(?i)\bmanage\.py\s+runserver\b"),
        ("PHP", r"\bphp\s+-S\s"),
        ("Wrangler", r"(?i)(?:^|[ /])wrangler(?:\.js)?\s+dev\b"),
    ]
    .into_iter()
    .map(|(name, p)| (name, Regex::new(p).expect("static regex")))
    .collect()
});
static AGENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|[ /_.-])(?:mcp|tsserver|language-server|sonarlint)(?:[ /_.-]|$)").unwrap()
});
static RUNTIME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(node|bun|deno|python[\d.]*|ruby|php|java|go|cargo|workerd|electron)$").unwrap()
});
pub fn assess(p: &ProcessInfo) -> Assessment {
    let exe = p.executable.as_deref().unwrap_or("");
    let binary = exe
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(&p.name)
        .to_lowercase();
    let name = p.name.to_lowercase();
    let has = |names: &[&str]| names.contains(&binary.as_str()) || names.contains(&name.as_str());
    if p.uid == 0
        || [
            "/System/Library/",
            "/usr/libexec/",
            "/usr/sbin/",
            "/Library/Apple/System/",
            "/Library/PrivilegedHelperTools/",
        ]
        .iter()
        .any(|prefix| exe.starts_with(prefix))
        || has(&[
            "launchd",
            "kernel_task",
            "controlcenter",
            "rapportd",
            "sharingd",
            "remotepairingd",
            "homed",
            "mdnsresponder",
            "configd",
            "apsd",
            "trustd",
            "airportd",
            "bluetoothd",
            "syspolicyd",
            "sshd",
            "systemd",
        ])
    {
        return Assessment::new(
            Kind::System,
            p.name.clone(),
            if p.uid == 0 {
                Evidence::Root
            } else {
                Evidence::System(if exe.is_empty() {
                    p.name.clone()
                } else {
                    exe.into()
                })
            },
        );
    }
    for (names, title) in [
        (&["postgres", "postmaster"][..], "PostgreSQL"),
        (&["mysqld"][..], "MySQL"),
        (&["mariadbd"][..], "MariaDB"),
        (&["redis", "redis-server"][..], "Redis"),
        (&["mongod"][..], "MongoDB"),
        (&["memcached"][..], "Memcached"),
    ] {
        if has(names) {
            return Assessment::new(
                Kind::Database,
                title.into(),
                Evidence::Database(p.name.clone()),
            );
        }
    }
    for (names, title) in [
        (&["com.docker.backend", "dockerd"][..], "Docker"),
        (&["containerd"][..], "containerd"),
        (&["colima"][..], "Colima"),
        (&["vpnkit"][..], "Docker network"),
        (&["nginx"][..], "Nginx"),
        (&["httpd"][..], "Apache"),
        (&["caddy"][..], "Caddy"),
    ] {
        if has(names) {
            return Assessment::new(
                Kind::Service,
                title.into(),
                Evidence::Service(p.name.clone()),
            );
        }
    }
    if let Some((prefix, _)) = exe.split_once(".app/Contents/") {
        let app = prefix.rsplit('/').next().unwrap_or(prefix);
        let runtime_app = (app == "Python" && exe.contains("/Python.framework/"))
            || (app == "Electron" && exe.contains("/node_modules/"));
        if !runtime_app {
            return Assessment::new(
                Kind::Application,
                app.into(),
                Evidence::Application(app.into()),
            );
        }
    }
    if AGENT.is_match(&p.command)
        || has(&["codex", "claude", "ollama"])
        || [&binary, &name]
            .iter()
            .any(|s| s.starts_with("language_server") || s.starts_with("logioptionsplus"))
        || [
            "/.devin/extensions/",
            "/.vscode/extensions/",
            "/.cursor/extensions/",
        ]
        .iter()
        .any(|s| exe.contains(s))
    {
        return Assessment::new(Kind::Application, p.name.clone(), Evidence::Agent);
    }
    if let Some((tool, _)) = WEB.iter().find(|(_, re)| re.is_match(&p.command)) {
        return Assessment::new(
            Kind::Web,
            p.project
                .as_ref()
                .map_or_else(|| tool.to_string(), |pr| format!("{} · {tool}", pr.name)),
            Evidence::Web(tool.to_string()),
        );
    }
    if let Some(pr) = &p.project
        && (RUNTIME.is_match(&binary) || RUNTIME.is_match(&name))
    {
        return Assessment::new(
            Kind::Development,
            pr.name.clone(),
            Evidence::Project {
                runtime: p.name.clone(),
                root: pr.root.clone(),
            },
        );
    }
    Assessment::new(Kind::Unknown, p.name.clone(), Evidence::Unknown)
}
