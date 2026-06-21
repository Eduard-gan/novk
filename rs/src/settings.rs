use loco_rs::config::Config;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub auth: Auth,
}

#[derive(Debug, Deserialize)]
pub struct Auth {
    #[serde(default = "default_cookie_name")]
    pub cookie_name: String,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            cookie_name: default_cookie_name(),
        }
    }
}

fn default_cookie_name() -> String {
    "auth".to_string()
}

impl Settings {
    pub fn from_config(config: &Config) -> Self {
        let Some(settings) = &config.settings else {
            return Self::default();
        };

        serde_json::from_value(settings.clone()).unwrap_or_default()
    }
}
