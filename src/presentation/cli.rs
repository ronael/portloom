use super::i18n::Locale;
use crate::{
    application::{preferences::Language, query::PortQuery},
    domain::model::Protocol,
};
use clap::{Args, CommandFactory, Parser, Subcommand};

fn language(value: &str) -> Result<Language, String> {
    Language::from_code(value).ok_or_else(|| "Choose en or fr.".into())
}
#[derive(Parser, Debug)]
#[command(
    name = "portloom",
    version,
    propagate_version = true,
    about = "Your ports, under control. Native monitor for macOS and Linux.",
    after_help = "No command: interactive interface.\nKeys: g projects · w HTTP check · o open · k stop · L language · ? help\nLanguage: --lang > PORTLOOM_LANG > saved preference > English.\nAn optional language picker appears on the first interactive launch."
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "Simulate without changing real processes or opening a browser"
    )]
    pub demo: bool,
    #[arg(
        long,
        global = true,
        help = "JSON output without an interactive interface"
    )]
    pub json: bool,
    #[arg(
        long,
        global = true,
        help = "Keep the mascot still (also toggle with a in the interface)"
    )]
    pub no_motion: bool,
    #[arg(long, global = true, default_value_t = 2000, value_parser = clap::value_parser!(u64).range(500..=60000), help = "Scan interval in milliseconds")]
    pub interval: u64,
    #[arg(long, global = true, value_parser = language, value_name = "en|fr", help = "Use this language once, without changing the saved preference")]
    pub lang: Option<Language>,
    #[command(subcommand)]
    pub command: Option<Command>,
}
#[derive(Args, Debug)]
pub struct TargetArgs {
    #[arg(value_parser = clap::value_parser!(u16).range(1..), help = "Local listening port")]
    pub port: u16,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..), help = "Choose a process when a port has multiple owners")]
    pub pid: Option<u32>,
    #[arg(long, value_parser = ["tcp", "udp"], help = "Restrict the transport protocol")]
    pub protocol: Option<String>,
}
impl TargetArgs {
    pub fn query(&self) -> PortQuery {
        PortQuery {
            port: Some(self.port),
            pid: self.pid,
            protocol: match self.protocol.as_deref() {
                Some("tcp") => Some(Protocol::Tcp),
                Some("udp") => Some(Protocol::Udp),
                _ => None,
            },
            projects_only: false,
        }
    }
}
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "List listening ports")]
    List {
        #[arg(long, help = "Only processes associated with development projects")]
        projects: bool,
    },
    #[command(about = "Inspect the owners and context of a port")]
    Inspect(TargetArgs),
    #[command(about = "Stop a process after reviewing all its ports and confirming")]
    Stop {
        #[command(flatten)]
        target: TargetArgs,
        #[arg(
            long,
            help = "Request SIGKILL explicitly; confirmation is still required"
        )]
        force: bool,
    },
    #[command(about = "Watch one port in the TUI, or stream JSON snapshots")]
    Watch {
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,
        #[arg(long, requires = "json", value_parser = clap::value_parser!(u32).range(1..=10000), help = "Stop after this many JSON snapshots")]
        count: Option<u32>,
    },
    #[command(about = "Verify HTTP on a local TCP listener (no redirects, 2 s limit)")]
    Probe(TargetArgs),
    #[command(about = "Verify local HTTP, then open the URL in your browser")]
    Open(TargetArgs),
    #[command(about = "Save the interface language for future launches")]
    Language {
        #[arg(value_parser = language, value_name = "en|fr")]
        language: Language,
    },
    #[command(about = "Generate shell completion")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
pub fn command(locale: Locale) -> clap::Command {
    let command = Cli::command();
    if locale != Locale::FR {
        return command;
    }
    let mut command = command
        .about("Tes ports, sous contrôle. Moniteur natif macOS et Linux.")
        .after_help("Sans commande : interface interactive.\nTouches : g projets · w vérifier HTTP · o ouvrir · k arrêter · L langue · ? aide\nLangue : --lang > PORTLOOM_LANG > préférence enregistrée > anglais.\nUn choix de langue est proposé au premier lancement interactif.")
        .mut_arg("demo", |a| a.help("Simulation sans agir sur les processus réels ni ouvrir le navigateur"))
        .mut_arg("json", |a| a.help("Sortie JSON sans interface interactive"))
        .mut_arg("no_motion", |a| a.help("Désactiver les animations de la mascotte (touche a dans l’interface)"))
        .mut_arg("interval", |a| a.help("Intervalle des relevés en millisecondes"))
        .mut_arg("lang", |a| a.help("Langue de ce lancement, sans modifier la préférence enregistrée"));
    for (name, about) in [
        ("list", "Lister les ports en écoute"),
        (
            "inspect",
            "Inspecter les propriétaires et le contexte d’un port",
        ),
        (
            "stop",
            "Arrêter un processus après vérification de tous ses ports et confirmation",
        ),
        (
            "watch",
            "Suivre un port dans l’interface ou en relevés JSON successifs",
        ),
        (
            "probe",
            "Vérifier HTTP sur un port TCP local, sans redirection, sous 2 s",
        ),
        (
            "open",
            "Vérifier HTTP local puis ouvrir l’URL dans le navigateur",
        ),
        (
            "language",
            "Enregistrer la langue pour les prochains lancements",
        ),
        ("completions", "Générer l’autocomplétion du shell"),
    ] {
        command = command.mut_subcommand(name, |c| c.about(about));
    }
    for name in ["inspect", "stop", "probe", "open"] {
        command = command.mut_subcommand(name, |c| {
            c.mut_arg("port", |a| a.help("Port local en écoute"))
                .mut_arg("pid", |a| {
                    a.help("Choisir le processus si le port a plusieurs propriétaires")
                })
                .mut_arg("protocol", |a| {
                    a.help("Restreindre le protocole de transport")
                })
        });
    }
    command
        .mut_subcommand("list", |c| {
            c.mut_arg("projects", |a| {
                a.help("Uniquement les processus de projets de développement")
            })
        })
        .mut_subcommand("stop", |c| {
            c.mut_arg("force", |a| {
                a.help("Demander SIGKILL explicitement, avec confirmation")
            })
        })
        .mut_subcommand("watch", |c| {
            c.mut_arg("count", |a| {
                a.help("Terminer après ce nombre de relevés JSON")
            })
        })
}
