use super::{
    assessment::{Kind, assess},
    model::ProcessInfo,
};
#[derive(Clone, Debug)]
pub struct Actor {
    pub uid: u32,
    pub protected_pids: Vec<u32>,
}
pub fn blocked(target: &ProcessInfo, actor: &Actor) -> Option<&'static str> {
    if target.pid <= 1 || target.pid > i32::MAX as u32 {
        return Some("Processus système protégé.");
    }
    if actor.protected_pids.contains(&target.pid) {
        return Some("Portloom et son terminal sont protégés.");
    }
    if target.uid != actor.uid {
        return Some("Ce processus appartient à un autre utilisateur.");
    }
    if assess(target).kind == Kind::System {
        return Some("Service système protégé : conserve ce processus.");
    }
    if target.started_at.is_empty() {
        return Some("Identité du processus indisponible. Actualise la liste.");
    }
    None
}
