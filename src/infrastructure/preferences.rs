use crate::application::{
    ports::AppResult,
    preferences::{Language, Preferences, PreferencesStore},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct FilePreferences {
    directory: Option<PathBuf>,
}
impl FilePreferences {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory: Some(directory),
        }
    }
    pub fn from_environment() -> Self {
        let directory = std::env::var_os("PORTLOOM_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("XDG_CONFIG_HOME").map(|p| PathBuf::from(p).join("portloom"))
            })
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config/portloom")))
            .filter(|p| p.is_absolute());
        Self { directory }
    }
    fn path(&self) -> AppResult<PathBuf> {
        self.directory
            .as_ref()
            .map(|dir| dir.join("settings.json"))
            .ok_or_else(|| {
                "No absolute configuration directory is available. Set PORTLOOM_CONFIG_DIR.".into()
            })
    }
    fn read(path: &Path) -> AppResult<serde_json::Value> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(serde_json::json!({})),
            Err(e) => return Err(format!("Cannot read language settings: {e}").into()),
        };
        let mut data = String::new();
        file.take(16_385)
            .read_to_string(&mut data)
            .map_err(|e| e.to_string())?;
        if data.len() > 16_384 {
            return Err("Language settings exceed 16 KiB.".into());
        }
        let value: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| format!("Invalid language settings: {e}"))?;
        if !value.is_object() {
            return Err("Language settings must be a JSON object.".into());
        }
        Ok(value)
    }
}
impl PreferencesStore for FilePreferences {
    fn load(&self) -> AppResult<Preferences> {
        if self.directory.is_none() {
            return Ok(Preferences::default());
        }
        let value = Self::read(&self.path()?)?;
        let language = match value.get("language") {
            None => None,
            Some(value) => Some(
                value
                    .as_str()
                    .and_then(Language::from_code)
                    .ok_or("Invalid saved language. Use portloom language en or fr.")?,
            ),
        };
        Ok(Preferences { language })
    }
    fn save_language(&self, language: Language) -> AppResult<()> {
        let path = self.path()?;
        let dir = path.parent().expect("settings parent");
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        // Explicit language changes can recover a malformed preference file.
        let mut value = Self::read(&path).unwrap_or_else(|_| serde_json::json!({}));
        value["language"] = serde_json::json!(language.code());
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = dir.join(format!(".settings-{}-{nonce}.tmp", std::process::id()));
        let result = (|| -> AppResult<()> {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary).map_err(|e| e.to_string())?;
            writeln!(
                file,
                "{}",
                serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?
            )
            .map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temporary, &path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
}
