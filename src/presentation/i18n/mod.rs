mod assessment;
mod catalog;
mod errors;
use crate::application::preferences::Language;
pub use catalog::Text as T;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Locale(pub Language);
impl Locale {
    pub const EN: Self = Self(Language::English);
    pub const FR: Self = Self(Language::French);
    pub fn format(self, key: T, arguments: &[(&str, &str)]) -> String {
        let mut result = self.text(key).to_owned();
        for (name, value) in arguments {
            result = result.replace(&format!("{{{name}}}"), value);
        }
        result
    }
    pub fn confirm_key(self) -> char {
        if self == Self::FR { 'o' } else { 'y' }
    }
    pub fn views(self, compact: bool) -> [&'static str; 6] {
        match (self, compact) {
            (Self::FR, false) => [
                "0 Tous",
                "1 Projets",
                "2 Services",
                "3 Apps",
                "4 Système",
                "5 Inconnus",
            ],
            (Self::FR, true) => ["0 Tout", "1 Proj", "2 Serv", "3 Apps", "4 Syst", "5 Inco"],
            (_, false) => [
                "0 All",
                "1 Projects",
                "2 Services",
                "3 Apps",
                "4 System",
                "5 Unknown",
            ],
            (_, true) => ["0 All", "1 Proj", "2 Serv", "3 Apps", "4 Sys", "5 Unkn"],
        }
    }
}
