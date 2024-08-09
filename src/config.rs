use log::debug;
use serde::Deserialize;
use std::fs;
use std::path::{PathBuf};

#[derive(Deserialize)]
pub struct General {
    pub wallpaper_dir: String,
    pub purity: Option<String>,
    pub wallpaper_app: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            wallpaper_dir: "".to_string(),
            purity: Some("sfw".to_string()),
            wallpaper_app: "".to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub database_path: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let mut config_path: PathBuf = dirs::home_dir().unwrap();
        config_path.push(".local/share/applications/sinh-x/wallpaper/");

        Self {
            database_path: config_path.to_str().unwrap().to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct Swww {}

#[derive(Deserialize)]
pub struct Feh {}

#[derive(Deserialize)]
pub struct Download {
    pub api_key: String,
    pub purity: String,
    pub categories: String,
    pub query: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub general: General,
    pub swww: Option<Swww>,
    pub feh: Option<Feh>,
    pub download: Download,
    pub database: Option<DatabaseConfig>,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, toml::de::Error> {
        let contents = fs::read_to_string(path).expect("Failed to read config file");
        let mut config: Config = toml::from_str(&contents)?;

        match &config.database {
            Some(db_config) => {
                debug!("Database path: {}", db_config.database_path);
            }
            None => {
                config.database = Some(DatabaseConfig::default());
                debug!(
                    "Database path is missing. Using default setting: {}",
                    "~/.local/share/applications/sinh-x/wallpaper/"
                );
            }
        }

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Add your validation logic here. For example:
        if self.download.api_key.is_empty() {
            return Err("API key is missing".to_string());
        }

        match self.general.wallpaper_app.as_str() {
            "swww" => {
                if self.swww.is_none() {
                    return Err("The 'swww' section is missing in the config file".to_string());
                }
            }
            "feh" => {
                if self.feh.is_none() {
                    return Err("The 'feh' section is missing in the config file".to_string());
                }
            }
            _ => return Err("Invalid wallpaper_app value. It must be 'swww' or 'feh'".to_string()),
        }

        Ok(())
    }
}
