use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct General {
    pub wallpaper_dir: String,
    pub wallpaper_app: String,
}

#[derive(Deserialize)]
pub struct Swww {}

#[derive(Deserialize)]
pub struct Feh {}

#[derive(Deserialize)]
pub struct Download {
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub general: General,
    pub swww: Option<Swww>,
    pub feh: Option<Feh>,
    pub download: Download,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, toml::de::Error> {
        let contents = fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&contents)
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
