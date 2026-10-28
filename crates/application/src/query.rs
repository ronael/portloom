use crate::domain::{
    assessment::{Kind, assess},
    issue::Issue,
    model::{Endpoint, Protocol, Snapshot},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct PortQuery {
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub protocol: Option<Protocol>,
    pub projects_only: bool,
}
impl PortQuery {
    pub fn matches(&self, endpoint: &Endpoint) -> bool {
        self.port.is_none_or(|p| p == endpoint.port)
            && self.pid.is_none_or(|p| p == endpoint.process.pid)
            && self.protocol.is_none_or(|p| p == endpoint.protocol)
            && (!self.projects_only || is_project(endpoint))
    }
    pub fn filter(&self, snapshot: Snapshot) -> Snapshot {
        Snapshot {
            captured_at: snapshot.captured_at,
            endpoints: snapshot
                .endpoints
                .into_iter()
                .filter(|e| self.matches(e))
                .collect(),
        }
    }
    pub fn target(&self, snapshot: &Snapshot) -> Result<Endpoint, Issue> {
        let mut rows = snapshot.endpoints.iter().filter(|e| self.matches(e));
        let first = rows
            .next()
            .ok_or(Issue::PortNotFound(self.port.unwrap_or(0)))?;
        if rows.any(|e| e.process.pid != first.process.pid) {
            return Err(Issue::AmbiguousPort(first.port));
        }
        Ok(first.clone())
    }
}
pub fn is_project(endpoint: &Endpoint) -> bool {
    endpoint.process.project.is_some()
        && matches!(
            assess(&endpoint.process).kind,
            Kind::Web | Kind::Development
        )
}
#[derive(Clone, Debug)]
pub struct ProjectGroup {
    pub root: String,
    pub name: String,
    pub endpoints: Vec<usize>,
}
/// Group by full project root, never by a possibly duplicated display name.
pub fn project_groups(snapshot: &Snapshot, indices: &[usize]) -> Vec<ProjectGroup> {
    let mut groups: BTreeMap<String, ProjectGroup> = BTreeMap::new();
    for &index in indices {
        let Some(endpoint) = snapshot.endpoints.get(index).filter(|e| is_project(e)) else {
            continue;
        };
        let project = endpoint.process.project.as_ref().expect("project endpoint");
        groups
            .entry(project.root.clone())
            .or_insert_with(|| ProjectGroup {
                root: project.root.clone(),
                name: project.name.clone(),
                endpoints: vec![],
            })
            .endpoints
            .push(index);
    }
    groups.into_values().collect()
}
