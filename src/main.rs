use dirs;
use rand::Rng;
use reqwest::Error;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

mod error;
use error::MyError;

mod config;
use config::Config;

#[derive(Deserialize)]
struct Wallpaper {
    id: String,
    url: String,
}

#[derive(StructOpt)]
enum Command {
    Download,
    Refresh,
    Setup,
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let command = Command::from_args();

    match command {
        Command::Download => download().await?,
        Command::Refresh => refresh()?,
        Command::Setup => setup()?,
    }

    Ok(())
}

async fn download() -> Result<(), MyError> {
    println!("Downloading wallpaper... not implemented yet.");

    let config = Config::new("/home/sinh/.config/sinh-x/wallpaper/config.toml")
        .expect("Failed to load config");
    config.validate().expect("Invalid config");

    // let api_key = "your_api_key";
    // let url = format!("https://wallhaven.cc/api/v1/search?apikey={}", api_key);
    // let response = reqwest::get(&url).await?.json::<Wallpaper>().await?;
    //
    // let path = Path::new("/path/to/local/directory").join(&response.id);
    // let bytes = reqwest::get(&response.url).await?.bytes().await?;
    // std::fs::write(&path, &bytes)?;
    //
    Ok(())
}

fn refresh() -> Result<(), MyError> {
    println!("Setting wallpaper...");

    let config = Config::new("/home/sinh/.config/sinh-x/wallpaper/config.toml")
        .expect("Failed to load config");
    config.validate().expect("Invalid config");

    match config.general.wallpaper_app.as_str() {
        "feh" => {
            println!("Setting wallpaper using feh...");
        }
        "swww" => {
            println!("Setting wallpaper using swww...");
        }
        &_ => {
            println!("Unknown wallpaper app");
        }
    }

    let entries = std::fs::read_dir(config.general.wallpaper_dir)?;
    let wallpapers: Vec<_> = entries.map(|e| e.unwrap().path()).collect();
    let mut rng = rand::thread_rng();
    let wallpaper = wallpapers[rng.gen_range(0..wallpapers.len())].clone();
    println!("Setting wallpaper: {}", wallpaper.display());

    // Set the wallpaper as the desktop background.
    // This depends on your desktop environment. For example, on GNOME:
    std::process::Command::new("swww")
        .arg("img")
        .arg(format!("{}", wallpaper.display()))
        .output()?;

    Ok(())
}

fn setup() -> Result<(), MyError> {
    println!("Setting up...");

    let mut config_path: PathBuf = dirs::home_dir().unwrap();
    config_path.push(".config/sinh-x/wallpaper");

    // Create the directory if it does not exist
    std::fs::create_dir_all(&config_path)?;

    config_path.push("config.toml");

    if !config_path.exists() {
        let mut file = File::create(&config_path)?;
        write!(file, "api_key = \"your_api_key\"\n")?;
    }

    Ok(())
}
