use super::Locale;
use crate::domain::issue::Issue;
impl Locale {
    pub fn error(self, issue: &Issue) -> String {
        if self != Self::FR {
            return issue.to_string();
        }
        match issue {
            Issue::SystemProcess => "Service système protégé : conserve ce processus.".into(),
            Issue::ProtectedProcess => "Portloom et son terminal sont protégés.".into(),
            Issue::OtherUser => "Ce processus appartient à un autre utilisateur.".into(),
            Issue::UnknownIdentity => {
                "Identité du processus indisponible. Actualise la liste.".into()
            }
            Issue::OwnerChanged => {
                "Ce port a changé de propriétaire. Actualise la sélection.".into()
            }
            Issue::ProcessChanged => "Le processus a changé. Aucun signal envoyé.".into(),
            Issue::UsageChanged => "L’usage du processus a changé. Confirme à nouveau.".into(),
            Issue::ExactPidRequired => {
                "Saisis le PID exact pour confirmer cet arrêt sensible.".into()
            }
            Issue::NewPorts => {
                "Ce processus a ouvert un autre port. Vérifie tous les ports et confirme à nouveau."
                    .into()
            }
            Issue::PidReused => "Le PID a été réutilisé. Aucun signal envoyé.".into(),
            Issue::PortNotFound(port) => format!("Aucun port en écoute correspondant à {port}."),
            Issue::AmbiguousPort(port) => {
                format!("Le port {port} a plusieurs propriétaires. Précise --pid.")
            }
            Issue::TcpRequired => "La vérification HTTP exige un port TCP en écoute.".into(),
            Issue::LoopbackRequired => {
                "La vérification HTTP se limite aux ports accessibles sur la boucle locale.".into()
            }
            Issue::HttpUnavailable => {
                "Aucune réponse HTTP sous 2 secondes. HTTPS n’est pas testé.".into()
            }
            Issue::InvalidHttp => "Le port n’a pas renvoyé de réponse HTTP/1.x valide.".into(),
            Issue::HttpHeaderTooLarge => "Les en-têtes HTTP dépassent la limite de 8 Kio.".into(),
            Issue::HttpOwnerAmbiguous => "Cette adresse HTTP locale pourrait appartenir à un autre processus sur ce port. Vérification bloquée.".into(),
            Issue::InteractiveRequired => {
                "Un terminal interactif est requis pour confirmer.".into()
            }
            Issue::Cancelled => "Annulé. Aucun signal envoyé.".into(),
            Issue::External(text) => text.clone(),
        }
    }
}
