use crate::domain::model::Project;
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};
type CachedProjects = BTreeMap<String, (Instant, Option<Project>)>;
#[derive(Default)]
pub struct ProjectFinder {
    cache: Mutex<CachedProjects>,
}
impl ProjectFinder {
    pub fn find(&self, cwd: &str) -> Option<Project> {
        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((time, value)) = cache.get(cwd)
            && time.elapsed() < Duration::from_secs(30)
        {
            return value.clone();
        }
        let value = discover(cwd, std::env::var("HOME").ok().as_deref());
        if cache.len() >= 256 {
            cache.pop_first();
        }
        cache.insert(cwd.into(), (Instant::now(), value.clone()));
        value
    }
}
pub fn discover(cwd: &str, home: Option<&str>) -> Option<Project> {
    if !Path::new(cwd).is_absolute() {
        return None;
    }
    for dir in Path::new(cwd).ancestors().take(5) {
        if dir.parent().is_none() || home.is_some_and(|h| dir == Path::new(h)) {
            break;
        }
        if dir.components().any(|c| {
            [
                "node_modules",
                ".npm",
                ".bun",
                ".local",
                ".cache",
                "Library",
            ]
            .iter()
            .any(|n| c.as_os_str() == *n)
        }) {
            return None;
        }
        let name = dir.file_name()?.to_string_lossy().into_owned();
        let project = |name| Project {
            name,
            root: dir.to_string_lossy().into_owned(),
            branch: branch(dir),
        };
        if let Ok(file) = fs::File::open(dir.join("package.json")) {
            let mut data = String::new();
            if file.take(128 * 1024 + 1).read_to_string(&mut data).is_ok()
                && data.len() <= 128 * 1024
                && let Ok(json) = serde_json::from_str::<serde_json::Value>(&data)
            {
                return Some(project(
                    json.get("name")
                        .and_then(|s| s.as_str())
                        .filter(|s| !s.trim().is_empty())
                        .unwrap_or(&name)
                        .into(),
                ));
            }
        }
        if [".git", "Cargo.toml", "pyproject.toml", "go.mod"]
            .iter()
            .any(|m| dir.join(m).exists())
        {
            return Some(project(name));
        }
    }
    None
}

fn small_file(path: &Path) -> Option<String> {
    let mut text = String::new();
    fs::File::open(path)
        .ok()?
        .take(4097)
        .read_to_string(&mut text)
        .ok()?;
    (text.len() <= 4096).then_some(text)
}
fn branch(root: &Path) -> Option<String> {
    for dir in root.ancestors().take(5) {
        let marker = dir.join(".git");
        let git = if marker.is_dir() {
            marker
        } else if marker.is_file() {
            let target = small_file(&marker)?;
            dir.join(target.trim().strip_prefix("gitdir: ")?)
        } else {
            continue;
        };
        let head = small_file(&git.join("HEAD"))?;
        let value = head.trim();
        if let Some(name) = value.strip_prefix("ref: refs/heads/") {
            return Some(name.into());
        }
        if value.len() >= 7 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Some(format!("detached:{}", &value[..7]));
        }
        return None;
    }
    None
}
