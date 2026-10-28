use super::ports::AppResult;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    French,
}
impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
        }
    }
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::English),
            "fr" => Some(Self::French),
            _ => None,
        }
    }
    pub fn suggest(locale: &str) -> Self {
        if locale
            .split(['_', '-', '.', ':'])
            .next()
            .is_some_and(|s| s.eq_ignore_ascii_case("fr"))
        {
            Self::French
        } else {
            Self::English
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Preferences {
    pub language: Option<Language>,
}
pub trait PreferencesStore: Send + Sync {
    fn load(&self) -> AppResult<Preferences>;
    fn save_language(&self, language: Language) -> AppResult<()>;
}
pub fn resolve(
    explicit: Option<Language>,
    environment: Option<Language>,
    saved: Option<Language>,
) -> Language {
    explicit.or(environment).or(saved).unwrap_or_default()
}
