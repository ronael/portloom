use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
pub struct Directory(pub PathBuf);
impl Directory {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "portloom-features-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub fn cli(&self, args: &[&str], env: Option<&str>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_portloom"));
        command
            .args(args)
            .env("PORTLOOM_CONFIG_DIR", &self.0)
            .env("LANG", "fr_FR.UTF-8")
            .env_remove("PORTLOOM_LANG");
        if let Some(value) = env {
            command.env("PORTLOOM_LANG", value);
        }
        command.output().unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
