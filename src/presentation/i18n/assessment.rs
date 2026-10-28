use super::Locale;
use crate::domain::assessment::{Assessment, Evidence, Kind};
pub struct AssessmentText {
    pub label: &'static str,
    pub badge: &'static str,
    pub reason: String,
    pub impact: &'static str,
}
impl Locale {
    pub fn assessment(self, a: &Assessment) -> AssessmentText {
        let fr = self == Self::FR;
        let (label, badge, impact) = match (a.kind, fr) {
            (Kind::Web, false) => (
                "Likely website / API",
                "WEB/API",
                "Stops this server and all its ports. Its pages or APIs will become unavailable.",
            ),
            (Kind::Web, true) => (
                "Site / API probable",
                "SITE/API",
                "Arrête ce serveur et tous ses ports. Les pages ou API associées deviendront indisponibles.",
            ),
            (Kind::Development, false) => (
                "Likely development project",
                "PROJECT",
                "Stops this project's process: server, worker, or development tool.",
            ),
            (Kind::Development, true) => (
                "Projet de développement probable",
                "PROJET",
                "Arrête ce processus du projet : serveur, worker ou outil de développement.",
            ),
            (Kind::Database, false) => (
                "Database / cache",
                "DATABASE",
                "Stops the database or cache. Several projects may depend on it.",
            ),
            (Kind::Database, true) => (
                "Base de données / cache",
                "BASE",
                "Arrête la base ou le cache. Plusieurs projets peuvent en dépendre.",
            ),
            (Kind::Service, false) if a.title.contains("Docker") => (
                "Shared service",
                "SERVICE",
                "This backend may serve several containers. Stop the relevant container in Docker instead.",
            ),
            (Kind::Service, true) if a.title.contains("Docker") => (
                "Service partagé",
                "SERVICE",
                "Ce backend peut servir plusieurs conteneurs. Préfère arrêter le conteneur concerné dans Docker.",
            ),
            (Kind::Service, false) => (
                "Shared service",
                "SERVICE",
                "May serve several projects or applications. Check its dependencies before stopping.",
            ),
            (Kind::Service, true) => (
                "Service partagé",
                "SERVICE",
                "Peut servir plusieurs projets ou applications. Vérifie ses dépendances avant de l’arrêter.",
            ),
            (Kind::Application, false) => (
                "Application / background tool",
                "APP",
                "May interrupt an application or active task. Prefer its own quit action.",
            ),
            (Kind::Application, true) => (
                "Application / outil en cours",
                "APPLI",
                "Peut interrompre une application ou une tâche. Préfère la quitter depuis son interface.",
            ),
            (Kind::System, false) => (
                "System service",
                "SYSTEM",
                "Keep running: networking, sharing, or system operation. Stopping is blocked in Portloom.",
            ),
            (Kind::System, true) => (
                "Service système",
                "SYSTÈME",
                "À conserver : réseau, partage ou fonctionnement du système. Arrêt bloqué dans Portloom.",
            ),
            (Kind::Unknown, false) => (
                "Purpose unknown",
                "UNKNOWN",
                "Review the command and directory before stopping. Another application may rely on this process.",
            ),
            (Kind::Unknown, true) => (
                "Usage à identifier",
                "INCONNU",
                "Consulte la commande et le dossier avant tout arrêt. Ce processus peut servir à une autre application.",
            ),
        };
        let reason = match (&a.evidence, fr) {
            (Evidence::Root, false) => "Process running as the system root user.".into(),
            (Evidence::Root, true) => "Processus exécuté par le compte système root.".into(),
            (Evidence::System(value), false) => format!("Recognized system program: {value}."),
            (Evidence::System(value), true) => format!("Programme système reconnu : {value}."),
            (Evidence::Database(value), false) => format!("Recognized database engine: {value}."),
            (Evidence::Database(value), true) => format!("Moteur reconnu : {value}."),
            (Evidence::Service(value), false) => format!("Recognized service: {value}."),
            (Evidence::Service(value), true) => format!("Service reconnu : {value}."),
            (Evidence::Application(value), false) => format!("Executable inside {value}.app."),
            (Evidence::Application(value), true) => format!("Exécutable rattaché à {value}.app."),
            (Evidence::Agent, false) => "Name or executable path identifies a background tool.".into(),
            (Evidence::Agent, true) => "Nom ou chemin correspondant à un outil de travail.".into(),
            (Evidence::Web(value), false) => format!("Recognized {value} command. HTTP verification is available on demand."),
            (Evidence::Web(value), true) => format!("Commande {value} reconnue. La vérification HTTP se fait à la demande."),
            (Evidence::Project { runtime, root }, false) => format!("Runtime {runtime} in project directory {root}."),
            (Evidence::Project { runtime, root }, true) => format!("Runtime {runtime} dans le dossier de projet {root}."),
            (Evidence::Unknown, false) => "Insufficient evidence. A runtime or port number does not prove this is a website.".into(),
            (Evidence::Unknown, true) => "Indices insuffisants. Le runtime ou le numéro du port ne prouvent pas qu’il s’agit d’un site.".into(),
        };
        AssessmentText {
            label,
            badge,
            impact,
            reason,
        }
    }
}
