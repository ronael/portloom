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
pub struct Assessment {
    pub kind: Kind,
    pub label: &'static str,
    pub badge: &'static str,
    pub title: String,
    pub reason: String,
    pub impact: &'static str,
    pub needs_pid: bool,
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
    let result =
        |kind, label, badge, title: String, reason: String, impact, needs_pid| Assessment {
            kind,
            label,
            badge,
            title,
            reason,
            impact,
            needs_pid,
        };
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
        return result(
            Kind::System,
            "Service système",
            "SYSTÈME",
            p.name.clone(),
            if p.uid == 0 {
                "Processus exécuté par le compte système root.".into()
            } else {
                format!(
                    "Programme système reconnu : {}.",
                    if exe.is_empty() { &p.name } else { exe }
                )
            },
            "À conserver : réseau, partage ou fonctionnement du système. Arrêt bloqué dans Portloom.",
            true,
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
            return result(
                Kind::Database,
                "Base de données / cache",
                "BASE",
                title.into(),
                format!("Moteur reconnu : {}.", p.name),
                "Arrête la base ou le cache. Plusieurs projets peuvent en dépendre.",
                true,
            );
        }
    }
    for (names, title) in [
        (&["com.docker.backend", "dockerd"][..], "Docker"),
        (&["containerd"][..], "Conteneurs"),
        (&["colima"][..], "Colima"),
        (&["vpnkit"][..], "Réseau Docker"),
        (&["nginx"][..], "Nginx"),
        (&["httpd"][..], "Apache"),
        (&["caddy"][..], "Caddy"),
    ] {
        if has(names) {
            return result(
                Kind::Service,
                "Service partagé",
                "SERVICE",
                title.into(),
                format!("Service reconnu : {}.", p.name),
                if title.contains("Docker") {
                    "Ce backend peut servir plusieurs conteneurs. Préfère arrêter le conteneur concerné dans Docker."
                } else {
                    "Peut servir plusieurs projets ou applications. Vérifie ses dépendances avant de l’arrêter."
                },
                true,
            );
        }
    }
    if let Some((prefix, _)) = exe.split_once(".app/Contents/") {
        let app = prefix.rsplit('/').next().unwrap_or(prefix);
        let runtime_app = (app == "Python" && exe.contains("/Python.framework/"))
            || (app == "Electron" && exe.contains("/node_modules/"));
        if !runtime_app {
            return result(
                Kind::Application,
                "Application installée",
                "APPLI",
                app.into(),
                format!("Exécutable rattaché à {app}.app."),
                "Peut interrompre une application ou une de ses fonctions. Préfère la quitter depuis son interface.",
                true,
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
        return result(
            Kind::Application,
            "Outil / agent en cours",
            "APPLI",
            p.name.clone(),
            "Nom ou chemin correspondant à un outil de travail.".into(),
            "Peut interrompre une tâche, un éditeur ou un périphérique en cours d’utilisation.",
            true,
        );
    }
    if let Some((tool, _)) = WEB.iter().find(|(_, re)| re.is_match(&p.command)) {
        return result(
            Kind::Web,
            "Site / API probable",
            "SITE/API",
            p.project
                .as_ref()
                .map_or_else(|| tool.to_string(), |pr| format!("{} · {tool}", pr.name)),
            format!("Commande {tool} reconnue. HTTP n’a pas été testé."),
            "Arrête ce serveur et tous ses ports. Les pages ou API associées deviendront indisponibles.",
            false,
        );
    }
    if let Some(pr) = &p.project
        && (RUNTIME.is_match(&binary) || RUNTIME.is_match(&name))
    {
        return result(
            Kind::Development,
            "Projet de développement probable",
            "PROJET",
            pr.name.clone(),
            format!("Runtime {} dans le dossier de projet {}.", p.name, pr.root),
            "Arrête ce processus du projet : serveur, worker ou outil de développement.",
            false,
        );
    }
    result(Kind::Unknown, "Usage à identifier", "INCONNU", p.name.clone(), "Indices insuffisants. Le runtime ou le numéro du port ne prouvent pas qu’il s’agit d’un site.".into(), "Consulte la commande et le dossier avant tout arrêt. Ce processus peut servir à une autre application.", true)
}
