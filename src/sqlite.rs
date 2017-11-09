use serde_json;
use xdg;
use errors::*;
use rusqlite;
use std::iter::FromIterator;
use std::str::FromStr;

pub struct Feed {
    pub id: u64,
    pub name: String,
    pub url: String,
    pub paused: bool,
    pub last_seen: u64,
}

pub struct Feeds {
    db: rusqlite::Connection
}

impl Feeds {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("rust2email").unwrap();
        let data_file = match path {
            Some(path) => path.into(),
            None => xdg_dirs.place_data_file("rust2email.db").unwrap(),
        };

        let db = rusqlite::Connection::open(data_file).unwrap();

        db.execute("CREATE TABLE IF NOT EXISTS misc (
            version     INTEGER DEFAULT 1
            )", &[]).unwrap();

        db.execute("CREATE TABLE IF NOT EXISTS feeds (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        STRING UNIQUE,
            url         STRING UNIQUE,
            paused      INTEGER DEFAULT 0,
            last_seen   INTEGER DEFAULT 0
            )", &[]).unwrap();

        db.execute("CREATE TABLE IF NOT EXISTS feeds_seen (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            parent_id   INTEGER,
            url         STRING UNIQUE,
            )", &[]).unwrap();

        Ok(Feeds { db })
    }

    pub fn close(&self) {
      self.db.close().unwrap();
    }

    pub fn contains(&self, name: &String) -> bool {
        let mut stmt = self.db.prepare("SELECT * FROM feeds WHERE name = ?1")
            .unwrap();
        return stmt.exists(&[name]).unwrap();
    }

    pub fn add_feed(&mut self, name: &String, url: &String) {
        self.db.execute("INSERT OR IGNORE INTO
        feeds (name, url) VALUES (?1, ?2)", &[name, url]).unwrap();
    }

    pub fn get_feeds(&self) -> Vec<Feed> {
        let mut stmt = self.db.prepare("SELECT * FROM feeds")
            .unwrap();

        let iter = stmt.query_map(&[], |row| {
        Feed {
            id: u64::from_str(row.get(0)).unwrap(),
            name: row.get(1),
            url: row.get(2),
            paused: row.get(3),
            last_seen: row.get(4),
        }
        }).unwrap();

        Vec::from_iter(iter)
    }

    pub fn get_active_feeds(&self) -> Vec<Feed> {
        let mut stmt = self.db.prepare("SELECT * FROM feeds WHERE paused == 0")
            .unwrap();

        let iter = stmt.query_map(&[], |row| {
        Feed {
            id: u64::from_str(row.get(0)).unwrap(),
            name: row.get(1),
            url: row.get(2),
            paused: row.get(3),
            last_seen: row.get(4),
        }
        }).unwrap();

        Vec::from_iter(iter)
    }

    pub fn pause(&self, indexes: Option<&Vec<u64>>) {
        if Some(indexes) = indexes {
            for index in indexes {
            self.db.execute("UPDATE feeds SET paused = 1 WHERE id = ?1", &[index]).unwrap();
            }
        } else {
            self.db.execute("UPDATE feeds SET paused = 1", &[]).unwrap();
        }
    }

    pub fn unpause(&self, indexes: Option<&Vec<u64>>) {
        if Some(indexes) = indexes {
            for index in indexes {
            self.db.execute("UPDATE feeds SET paused = 0 WHERE id = ?1", &[index]).unwrap();
            }
        } else {
            self.db.execute("UPDATE feeds SET paused = 0", &[]).unwrap();
        }
    }

    pub fn delete(&self, indexes: Option<&Vec<u64>>) {
        if Some(indexes) = indexes {
            for index in indexes {
            self.db.execute("DELETE FROM feeds_seen WHERE parent_id = ?1", &[index]).unwrap();
            self.db.execute("DELETE FROM feeds WHERE id = ?1", &[index]).unwrap();
            }
        } else {
            self.db.execute("DELETE FROM feeds_seen", &[]).unwrap();
            self.db.execute("DELETE FROM feeds", &[]).unwrap();
        }
    }

    pub fn reset(&self, indexes: Option<&Vec<u64>>) {
        if Some(indexes) = indexes {
            for index in indexes {
            self.db.execute("DELETE FROM feeds_seen WHERE parent_id = ?1", &[index]).unwrap();
            }
        } else {
            self.db.execute("DELETE FROM feeds_seen", &[]).unwrap();
        }
    }

    pub fn has_been_seen(&self, feed_id: u64, entry_id: &String) -> bool {
        let mut stmt = self.db.prepare("SELECT * FROM feeds_seen WHERE parent_id = ?1 AND url = ?2")
            .unwrap();
        return stmt.exists(&[feed_id, entry_id]).unwrap();
    }

    pub fn see(&self, feed_id: u64, entry_id: &String) {
        self.db.execute("INSERT OR IGNORE INTO feeds_seen SET (parent_id, url) VALUES (?1, ?2)",
        &[feed_id, entry_id]).unwrap();
    }
}
