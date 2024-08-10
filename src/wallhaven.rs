use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_str;
use std::fs;
use std::path::{Path, PathBuf};

use crate::database::Database;
use crate::error::MyError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Wallpaper {
    pub id: String,
    pub url: String,
    pub short_url: String,
    pub views: u32,
    pub favorites: u32,
    pub source: String,
    pub purity: String,
    pub category: String,
    pub dimension_x: u32,
    pub dimension_y: u32,
    pub resolution: String,
    pub ratio: String,
    pub file_size: u32,
    pub file_type: String,
    pub created_at: String,
    pub colors: Vec<String>,
    pub path: String,
    pub thumbs: Thumbs,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Thumbs {
    pub large: String,
    pub original: String,
    pub small: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Meta {
    pub current_page: u32,
    pub last_page: u32,
    pub per_page: String,
    pub total: u32,
    pub query: Option<String>,
    pub seed: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub data: Vec<Wallpaper>,
    pub meta: Meta,
}

pub struct WallHaven {
    api_key: String,
    purity: String,
    categories: String,
    atleast: String,
    query: String,
    download_location: String,
    db: Database,
}

impl WallHaven {
    pub fn new(
        api_key: &str,
        purity: &str,
        categories: &str,
        atleast: &str,
        query: &str,
        download_location: &str,
        db: &Database,
    ) -> Self {
        Self {
            api_key: api_key.to_string(),
            purity: purity.to_string(),
            categories: categories.to_string(),
            atleast: atleast.to_string(),
            query: query.to_string(),
            download_location: download_location.to_string(),
            db: db.clone(),
        }
    }

    pub async fn download(&self) -> Result<(), MyError> {
        println!("Downloading wallpaper...");

        let db = &self.db;
        let api_key = &self.api_key;
        let purity = &self.purity; // Replace with the actual purity value
        let categories = &self.categories; // Replace with the actual category value
        let atleast = &self.atleast;
        let query = &self.query;
        let download_location = &self.download_location;

        let pb = ProgressBar::new(10);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .expect("Failed to create progress bar style"),
        );
        //

        let mut page = 1;
        let mut total_pages = 1;
        let mut count = 0;
        let mut sfw = 0;
        let mut nsfw = 0;
        while count < 10 {
            let url = format!(
            "https://wallhaven.cc/api/v1/search?apikey={}&purity={}&categories={}&page={}&atleast={}&q={}",
            api_key, purity, categories, page, atleast, query
        );
            debug!("URL: {}", &url);
            let response_text = reqwest::get(&url).await?.text().await?;

            // Parse the JSON response into a Response instance
            let response: Response = from_str(&response_text)?;
            if page == 1 {
                total_pages = response.meta.last_page;
                println!("Total wallpapers to searched: {}", response.meta.total);
                println!("Listing pages: {}", response.meta.last_page);
                pb.tick(); // Redraw the progress bar immediately
            }
            for wallpaper in response.data {
                let wallpaper_dir = download_location;
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

                match db.get_wallpaper_details(&file_name) {
                    Ok(_) => {
                        debug!("Wallpaper already exists in the database");
                        continue;
                    }
                    Err(_) => {
                        let _ = db.save_to_db(&file_name, &wallpaper);

                        let mut file_exists = false;

                        for folder_path in folder_paths {
                            let full_path = Path::new(&folder_path).join(&file_name);
                            if full_path.exists() {
                                file_exists = true;
                                break;
                            }
                        }

                        if !file_exists {
                            let mut file_path = PathBuf::from(download_location);
                            if wallpaper.purity != "sfw" {
                                nsfw += 1;
                                file_path = file_path.join("nsfw");
                            } else {
                                sfw += 1;
                            }

                            file_path = file_path.join(&file_name);

                            if !file_path.exists() {
                                let image_bytes =
                                    reqwest::get(&wallpaper.path).await?.bytes().await?;
                                fs::write(&file_path, image_bytes)?;
                                count += 1;
                                pb.inc(1);

                                debug!("Saved wallpaper to: {:?}", file_path);
                                debug!("Current count: {}", count);

                                if count >= 10 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            page += 1;
            if page > response.meta.last_page {
                break;
            }
        }

        println!(
            "Sfw: {} --- Nsfw {} --- reached: {}/{}",
            sfw, nsfw, page, total_pages
        );
        Ok(())
    }
}
