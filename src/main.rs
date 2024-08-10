use dirs;
use log::debug;
use rand::Rng;
use regex::Regex;
use serde_json::Value;
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
mod database;
mod wallhaven;

use config::Config;
use database::Database;
use wallhaven::Wallpaper;

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
    Current,
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    let opt = Opt::from_args();

    let config_path: PathBuf = dirs::home_dir().unwrap();
    let config_path = config_path.join(".config/sinh-x/wallpaper/config.toml");
    let config = Config::new(&config_path.display().to_string()).expect("Failed to load config");
    config.validate().expect("Invalid config");

    let binding = &config.database.unwrap();
    let db_path = Path::new(&binding.database_path);
    let db = Database::new(&db_path).unwrap();

    match opt.cmd {
        Command::Refresh { path } => refresh(path.as_deref())?,
        Command::Download => {
            let wallhaven = wallhaven::WallHaven::new(
                &config.download.api_key,
                &config.download.purity,
                &config.download.categories,
                &"2880x1800",
                &config.download.query,
                &config.general.wallpaper_dir,
                &db,
            );

            match wallhaven.download().await {
                Ok(_) => println!("Downloaded wallpapers successfully"),
                Err(e) => eprintln!("Failed to download wallpapers: {}", e),
            }
            match db.load_from_db() {
                Ok(wallpapers) => {
                    for wallpaper in &wallpapers {
                        debug!("{:?}", wallpaper.url);
                    }
                    println!("Total {} wallpapers in the database", wallpapers.len());
                }
                Err(e) => eprintln!("Failed to load wallpapers from the database: {}", e),
            }
        }
        Command::Setup => setup()?,
        Command::Archive { dir, archive_dir } => {
            let dir = dir.unwrap_or_else(|| PathBuf::from(&config.general.wallpaper_dir));
            let archive_dir = archive_dir
                .unwrap_or_else(|| PathBuf::from(&config.general.wallpaper_dir).join("archive"));
            archive(dir, archive_dir)?;
        }
        Command::Current => {
            // Read the ~/.fehbg file
            let fehbg = fs::read_to_string(Path::new(&format!(
                "{}/.fehbg",
                dirs::home_dir().unwrap().to_str().unwrap()
            )))
            .unwrap();

            // Extract the wallpaper path from the fehbg file
            let re = Regex::new(r"'(.*?)'").unwrap();
            let caps = re.captures(&fehbg).unwrap();
            let path = caps.get(1).map_or("", |m| m.as_str());

            // Extract the file name from the wallpaper path
            let path = Path::new(path);
            let file_name = path.file_name().unwrap().to_str().unwrap();

            let wallpaper = db.get_wallpaper_details(file_name).unwrap();

            println!("{}", file_name);

            println!("{}", wallpaper.id);

            let client = reqwest::Client::new();
            let url = format!("https://wallhaven.cc/api/v1/w/{}", wallpaper.id);
            let res = client.get(url).send().await?;
            let body: Value = res.json().await?;
            let tags = &body["data"]["tags"];
            for tag in tags.as_array().unwrap() {
                println!("{}", tag["name"].as_str().unwrap());
            }
        }
    }

    Ok(())
}

fn refresh(path: Option<&Path>) -> Result<(), MyError> {
    println!("Setting wallpaper...");

    let config = Config::new("/home/sinh/.config/sinh-x/wallpaper/config.toml")
        .expect("Failed to load config");
    config.validate().expect("Invalid config");

    let mut wallpaper_dir = PathBuf::from(&config.general.wallpaper_dir);
    if config.general.purity.as_deref() == Some("nsfw") {
        wallpaper_dir = wallpaper_dir.join("nsfw");
    }

    let wallpaper = match path {
        Some(path) => path.to_path_buf(),
        None => {
            let entries = std::fs::read_dir(wallpaper_dir)?;
            let wallpapers: Vec<_> = entries
                .filter_map(Result::ok)
                .filter(|e| e.path().is_file())
                .map(|e| e.path())
                .collect();
            let mut rng = rand::thread_rng();
            wallpapers[rng.gen_range(0..wallpapers.len())].clone()
        }
    };
    println!("Setting wallpaper: {}", wallpaper.display());

    match config.general.wallpaper_app.as_str() {
        "feh" => {
            println!("Setting wallpaper using feh...");
            let output = std::process::Command::new("feh")
                .arg("--bg-max")
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
    } else {
        let config = fs::read_to_string(&config_path)?;
        println!("Config: {}", config);
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
