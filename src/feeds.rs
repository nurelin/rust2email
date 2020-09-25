use crate::errors::*;
use serde_json;
use std::collections::HashSet;
use std::fs::{rename, OpenOptions};
use std::io::{Read, Write};
use xdg;

#[derive(Debug, Deserialize, Serialize)]
pub struct Feed {
    pub name: String,
    pub url: String,
    pub paused: bool,
    pub seen: HashSet<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Feeds {
    pub version: u32,
    pub feeds: Vec<Feed>,
}

impl Feeds {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("rust2email").unwrap();
        let data_file = match path {
            Some(path) => path.into(),
            None => xdg_dirs.place_data_file("rust2email.json").unwrap(),
        };

        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(data_file)
            .unwrap();

        let mut config = Feeds {
            version: 1,
            feeds: Vec::new(),
        };

        if f.metadata().unwrap().len() != 0 {
            let mut data = String::new();
            f.read_to_string(&mut data).unwrap();

            config = serde_json::from_str(data.as_str()).unwrap();
        }

        Ok(config)
    }

    pub fn save(&self, path: Option<&str>) -> Result<()> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("rust2email").unwrap();
        let (data_file, data_file_tmp) = match path {
            Some(path) => {
                let data_tmp = format!("{}.new", &path);
                (path.into(), data_tmp.into())
            }
            None => (
                xdg_dirs.place_data_file("rust2email.json").unwrap(),
                xdg_dirs.place_data_file("rust2email.json.new").unwrap(),
            ),
        };

        {
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&data_file_tmp)
                .unwrap();

            let data = serde_json::to_string_pretty(&self).unwrap();

            f.write(data.as_bytes()).unwrap();
        }

        rename(data_file_tmp, data_file).unwrap();

        Ok(())
    }

    pub fn contains(&self, name: &str) -> bool {
        for ref feed in &self.feeds {
            if feed.name == name {
                return true;
            }
        }
        return false;
    }

    pub fn push(&mut self, name: &str, url: &str) {
        self.feeds.push(Feed {
            name: name.to_string(),
            url: url.to_string(),
            paused: false,
            seen: HashSet::new(),
        });
    }
}
