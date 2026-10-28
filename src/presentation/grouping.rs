use crate::{
    application::query::{ProjectGroup, is_project, project_groups},
    domain::model::Snapshot,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub enum VisibleRow {
    Group(ProjectGroup),
    Endpoint(usize),
}
impl VisibleRow {
    pub fn key(&self, snapshot: &Snapshot) -> String {
        match self {
            Self::Group(group) => format!("project:{}", group.root),
            Self::Endpoint(index) => format!("endpoint:{}", snapshot.endpoints[*index].id),
        }
    }
}
pub fn rows(
    snapshot: &Snapshot,
    visible: &[usize],
    grouped: bool,
    collapsed: &BTreeSet<String>,
    searching: bool,
) -> Vec<VisibleRow> {
    if !grouped {
        return visible.iter().map(|i| VisibleRow::Endpoint(*i)).collect();
    }
    let mut rows = vec![];
    for group in project_groups(snapshot, visible) {
        let expanded = searching || !collapsed.contains(&group.root);
        let children = group.endpoints.clone();
        rows.push(VisibleRow::Group(group));
        if expanded {
            rows.extend(children.into_iter().map(VisibleRow::Endpoint));
        }
    }
    rows.extend(
        visible
            .iter()
            .filter(|i| !is_project(&snapshot.endpoints[**i]))
            .map(|i| VisibleRow::Endpoint(*i)),
    );
    rows
}
