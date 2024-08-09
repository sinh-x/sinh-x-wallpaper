use bincode;
// use colored::*;
use log::debug;
use sled::Db;
use std::path::Path;

use crate::Wallpaper;

#[derive(Debug)]
pub enum DatabaseError {
    KeyNotExist,
    SledError(sled::Error),
    BinCodeError(bincode::Error),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DatabaseError::KeyNotExist => write!(f, "Key not existed in database."),
            DatabaseError::SledError(err) => write!(f, "sled error: {}", err),
            DatabaseError::BinCodeError(err) => write!(f, "bincode error: {}", err),
        }
    }
}

impl From<sled::Error> for DatabaseError {
    fn from(err: sled::Error) -> DatabaseError {
        DatabaseError::SledError(err)
    }
}
impl From<bincode::Error> for DatabaseError {
    fn from(err: bincode::Error) -> DatabaseError {
        DatabaseError::BinCodeError(err)
    }
}

pub struct Database {
    db: Db,
    summary_db: Db,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            summary_db: self.summary_db.clone(),
        }
    }
}

impl Database {
    pub fn new(path: &Path) -> Result<Self, DatabaseError> {
        let _ = std::fs::create_dir_all(path);
        let db = sled::open(path.join("wallpaper_db"))?;
        let summary_db = sled::open(path.join("summary_db"))?;
        Ok(Self { db, summary_db })
    }

    pub fn save_to_db(&self, filename: &str, wallpaper: &Wallpaper) -> Result<(), DatabaseError> {
        self.db
            .insert(filename.as_bytes(), bincode::serialize(wallpaper)?)?;
        Ok(())
    }

    //TODO: Review this function
    #[allow(dead_code)]
    pub fn load_from_db(&self) -> Result<Vec<Wallpaper>, DatabaseError> {
        let mut repos = Vec::new();
        for result in self.db.iter() {
            let (_key, value) = result?;
            let repo: Wallpaper = bincode::deserialize(&value)?;
            repos.push(repo);
        }
        Ok(repos)
    }

    pub fn get_wallpaper_details(&self, key: &str) -> Result<Wallpaper, DatabaseError> {
        match self.db.get(key.to_string()) {
            Ok(Some(value)) => match bincode::deserialize(&value) {
                Ok(wallpaper) => return Ok(wallpaper),
                Err(e) => return Err(DatabaseError::BinCodeError(e)),
            },
            Ok(None) => return Err(DatabaseError::KeyNotExist),
            Err(e) => {
                debug!("git_wallpaper_details: data handling error!");
                return Err(DatabaseError::SledError(e));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Response;
    use lazy_static::lazy_static;
    use std::fs;
    use std::sync::Once;

    static INIT: Once = Once::new();
    static TEST_DB_PATH: &str = "/tmp/sinh-x_gitstatus-test.db";

    lazy_static! {
        static ref RESPONSE: Response = {
            let response_text = fs::read_to_string("data/wallhaven_test_response.json").unwrap();
            serde_json::from_str(&response_text).unwrap()
        };
    }

    fn setup() -> Database {
        INIT.call_once(|| {
            let _ = fs::remove_dir_all(TEST_DB_PATH); // Delete the test database if it exists
        });
        Database::new(Path::new(TEST_DB_PATH)).unwrap()
    }

    #[test]
    fn test_gitdatabase() {
        let db = setup();

        let wallpaper = RESPONSE.data[0].clone();
        let file_name = format!(
            "wallhaven-{}-{}.{}",
            wallpaper.id,
            wallpaper.resolution,
            wallpaper.file_type.split('/').last().unwrap()
        );

        db.save_to_db(&file_name, &wallpaper).unwrap();

        // Verify that the repo was saved correctly
        let wallpapers = db.load_from_db().unwrap();
        assert_eq!(
            wallpapers.len(),
            1,
            "1::Expected 1 repo, but found {}",
            wallpapers.len()
        );
        assert_eq!(
            wallpapers[0], wallpaper,
            "1::Saved repo does not match loaded repo"
        );

        let wallpaper = RESPONSE.data[1].clone();
        db.save_to_db(&file_name, &wallpaper).unwrap();

        // Verify that the repo was saved correctly
        let wallpapers = db.load_from_db().unwrap();
        assert_eq!(
            wallpapers.len(),
            1,
            "2::Expected 1 repo, but found {}",
            wallpapers.len()
        );
        assert_eq!(
            wallpapers[0], wallpaper,
            "2::Saved repo does not match loaded repo"
        );

        let wallpaper = RESPONSE.data[2].clone();
        let file_name = format!(
            "wallhaven-{}-{}.{}",
            wallpaper.id,
            wallpaper.resolution,
            wallpaper.file_type.split('/').last().unwrap()
        );

        db.save_to_db(&file_name, &wallpaper).unwrap();

        // Verify that the repo was saved correctly
        let wallpapers = db.load_from_db().unwrap();
        assert_eq!(
            wallpapers.len(),
            2,
            "3::Expected 2 wallpapers, but found {}",
            wallpapers.len()
        );
        assert_eq!(
            wallpapers[1], wallpaper,
            "3::Saved repo does not match loaded repo"
        );

        let _ = fs::remove_dir_all(TEST_DB_PATH);
    }
}
