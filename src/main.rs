use dirs;
use rand::Rng;
use reqwest::Error;
use serde::Deserialize;
use serde_json::from_str;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use structopt::StructOpt;

mod error;
use error::MyError;

mod config;
use config::Config;

#[derive(Deserialize, Debug)]
struct Wallpaper {
    id: String,
    url: String,
    short_url: String,
    views: u32,
    favorites: u32,
    source: String,
    purity: String,
    category: String,
    dimension_x: u32,
    dimension_y: u32,
    resolution: String,
    ratio: String,
    file_size: u32,
    file_type: String,
    created_at: String,
    colors: Vec<String>,
    path: String,
    thumbs: Thumbs,
}

#[derive(Deserialize, Debug)]
struct Thumbs {
    large: String,
    original: String,
    small: String,
}

#[derive(Deserialize, Debug)]
struct Meta {
    current_page: u32,
    last_page: u32,
    per_page: String,
    total: u32,
    query: Option<String>,
    seed: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Response {
    data: Vec<Wallpaper>,
    meta: Meta,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "wallpaper")]
struct Opt {
    /// Command to execute
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Refresh the wallpaper
    Refresh {
        /// Path to the wallpaper
        #[structopt(short, long, parse(from_os_str))]
        path: Option<PathBuf>,
    },
    Download,
    Setup,
    Archive {
        #[structopt(short, long, parse(from_os_str))]
        dir: Option<PathBuf>,
        #[structopt(short, long, parse(from_os_str))]
        archive_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let opt = Opt::from_args();

    let config = Config::new("/home/sinh/.config/sinh-x/wallpaper/config.toml")
        .expect("Failed to load config");
    config.validate().expect("Invalid config");

    match opt.cmd {
        Command::Refresh { path } => refresh(path.as_deref())?,
        Command::Download => download().await?,
        Command::Setup => setup()?,
        Command::Archive { dir, archive_dir } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from(&config.general.wallpaper_dir));
            let archive_dir = archive_dir
                .unwrap_or_else(|| PathBuf::from(&config.general.wallpaper_dir).join("archive"));
            archive(dir, archive_dir)?;
        }
    }

    Ok(())
}

async fn download() -> Result<(), MyError> {
    println!("Downloading wallpaper...");

    let config = Config::new("/home/sinh/.config/sinh-x/wallpaper/config.toml")
        .expect("Failed to load config");
    config.validate().expect("Invalid config");

    let api_key = config.download.api_key;
    let purity = config.download.purity; // Replace with the actual purity value
    let categories = config.download.categories; // Replace with the actual category value
    let mut page = 1;
    let mut count = 0;
    let atleast = "2880x1800";
    let query = config.download.query;
    while count < 10 {
        let url = format!(
            "https://wallhaven.cc/api/v1/search?apikey={}&purity={}&categories={}&page={}&atleast={}&q={}",
            api_key, purity, categories, page, atleast, query
        );
        println!("URL: {}", &url);
        let response_text = reqwest::get(&url).await?.text().await?;

        // Parse the JSON response into a Response instance
        let response: Response = from_str(&response_text)?;

        for wallpaper in response.data {
            let wallpaper_dir = &config.general.wallpaper_dir;
            let folder_paths: Vec<_> = fs::read_dir(wallpaper_dir)
                .expect("Directory not found")
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    if entry.file_type().ok()?.is_dir() {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
                .collect();

            let file_name = format!(
                "wallhaven-{}-{}.{}",
                wallpaper.id,
                wallpaper.resolution,
                wallpaper.file_type.split('/').last().unwrap()
            );

            let mut file_exists = false;

            for folder_path in folder_paths {
                let full_path = Path::new(&folder_path).join(&file_name);
                if full_path.exists() {
                    file_exists = true;
                    break;
                }
            }

            if !file_exists {
                let mut file_path = PathBuf::from(&config.general.wallpaper_dir);
                if wallpaper.purity != "sfw" {
                    file_path = file_path.join("nsfw");
                }

                file_path = file_path.join(&file_name);

                if !file_path.exists() {
                    let image_bytes = reqwest::get(&wallpaper.path).await?.bytes().await?;
                    fs::write(&file_path, image_bytes)?;
                    count += 1;

                    println!("Saved wallpaper to: {:?}", file_path);
                    println!("Current count: {}", count);

                    if count >= 10 {
                        break;
                    }
                }
            }
        }

        page += 1;
    }

    Ok(())
}

fn refresh(path: Option<&Path>) -> Result<(), MyError> {
    println!("Setting wallpaper...");

    let config = Config::new("/home/sinh/.config/sinh-x/wallpaper/config.toml")
        .expect("Failed to load config");
    config.validate().expect("Invalid config");

    let wallpaper = match path {
        Some(path) => path.to_path_buf(),
        None => {
            let entries = std::fs::read_dir(config.general.wallpaper_dir)?;
            let wallpapers: Vec<_> = entries.map(|e| e.unwrap().path()).collect();
            let mut rng = rand::thread_rng();
            wallpapers[rng.gen_range(0..wallpapers.len())].clone()
        }
    };
    println!("Setting wallpaper: {}", wallpaper.display());

    match config.general.wallpaper_app.as_str() {
        "feh" => {
            println!("Setting wallpaper using feh...");
            let output = std::process::Command::new("feh")
                .arg("--bg-fill")
                .arg("--image-bg #000000")
                .arg(format!("{}", wallpaper.display()))
                .output()?;

            let command_str = format!("feh --bg-scale {}", wallpaper.display());
            println!("Command: {}", command_str);

            if output.status.success() {
                println!("Wallpaper set successfully");
            } else {
                eprintln!(
                    "Failed to set wallpaper: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        "swww" => {
            println!("Setting wallpaper using swww...");
            // Set the wallpaper as the desktop background.
            // This depends on your desktop environment. For example, on GNOME:
            let output = std::process::Command::new("swww")
                .arg("img")
                .arg(format!("{}", wallpaper.display()))
                .output()?;
            let command_str = format!("swww img {}", wallpaper.display());
            println!("Command: {}", command_str);

            if output.status.success() {
                println!("Wallpaper set successfully");
            } else {
                eprintln!(
                    "Failed to set wallpaper: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        &_ => {
            println!("Unknown wallpaper app");
        }
    }

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

fn archive(dir: PathBuf, archive_dir: PathBuf) -> std::io::Result<()> {
    let one_week_ago = SystemTime::now() - Duration::from_secs(60 * 60 * 24 * 7);

    fs::create_dir_all(&archive_dir)?;

    let mut archived_files = 0;

    fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            if metadata.is_file() && metadata.modified().ok()? < one_week_ago {
                Some(entry.path())
            } else {
                None
            }
        })
        .for_each(|old_file_path| {
            let archive_file_path = archive_dir.join(old_file_path.file_name().unwrap());
            fs::rename(old_file_path, archive_file_path).unwrap();
            archived_files += 1;
        });

    let total_files = fs::read_dir(&archive_dir)?
        .filter(|entry| {
            entry
                .as_ref()
                .ok()
                .and_then(|e| e.metadata().ok())
                .map_or(false, |m| m.is_file())
        })
        .count();

    println!(
        "Archived {} files. Total files in archive: {}",
        archived_files, total_files
    );

    Ok(())
}
