use clap::{Parser, Subcommand};
#[derive(Parser, Debug)]
#[command(
    name = "portloom",
    version,
    about = "Tes ports, sous contrôle. Moniteur natif macOS et Linux.",
    after_help = "Sans argument : interface interactive.\nVues : 0 tous · 1 projets · 2 services · 3 apps · 4 système · 5 inconnus\nClavier : ↑↓ choisir · / chercher · Tab protocole · k arrêter · K forcer\n          Entrée détail · espace pause · r actualiser · ? aide · q quitter"
)]
pub struct Cli {
    #[arg(long, help = "Démonstration sans agir sur tes processus")]
    pub demo: bool,
    #[arg(long, help = "Un relevé JSON sans interface ni couleur")]
    pub json: bool,
    #[arg(long, default_value_t = 2000, value_parser = clap::value_parser!(u64).range(500..=60000), help = "Intervalle de scan en millisecondes")]
    pub interval: u64,
    #[command(subcommand)]
    pub command: Option<Command>,
}
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "Générer l’autocomplétion pour ton shell")]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
