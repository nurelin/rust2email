use crate::APP_INFO;
use failure::Error;
use fs2::FileExt;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;

pub struct Database {
    pub file: File,
    pub data: DatabaseData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseData {
    pub seens: HashMap<String, HashSet<String>>,
}

impl Database {
    pub fn new(possible_path: &Option<PathBuf>) -> Result<Self, Error> {
        let filepath: PathBuf = match possible_path {
            Some(path) => path.into(),
            None => match app_dirs::app_root(app_dirs::AppDataType::UserData, &APP_INFO) {
                Ok(path) => path.join("rust2email.db"),
                Err(err) => panic!(err),
            },
        };

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(filepath)?;
        file.try_lock_exclusive()?;
        // TODO check lock

        let data: DatabaseData = serde_json::from_reader(&file).unwrap_or(DatabaseData {
            seens: HashMap::new(),
        });

        Ok(Database { file, data })
    }

    pub fn update(&mut self, url: &str, new_seen_feeds: HashSet<String>) -> Result<(), Error> {
        self.data.seens.insert(url.to_string(), new_seen_feeds);
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        serde_json::to_writer(&self.file, &self.data)?;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<(), Error> {
        self.file.unlock()?;
        Ok(())
    }
}
