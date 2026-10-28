use super::{
    assessment::{Kind, assess},
    issue::Issue,
    model::ProcessInfo,
};
#[derive(Clone, Debug)]
pub struct Actor {
    pub uid: u32,
    pub protected_pids: Vec<u32>,
}
pub fn blocked(target: &ProcessInfo, actor: &Actor) -> Option<Issue> {
    if target.pid <= 1 || target.pid > i32::MAX as u32 {
        return Some(Issue::SystemProcess);
    }
    if actor.protected_pids.contains(&target.pid) {
        return Some(Issue::ProtectedProcess);
    }
    if target.uid != actor.uid {
        return Some(Issue::OtherUser);
    }
    if assess(target).kind == Kind::System {
        return Some(Issue::SystemProcess);
    }
    if target.started_at.is_empty() {
        return Some(Issue::UnknownIdentity);
    }
    None
}
