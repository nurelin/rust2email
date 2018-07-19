use lettre::file::FileEmailTransport;
use lettre::sendmail::SendmailTransport;
use lettre::EmailTransport;

use std::str::FromStr;

use settings::{MailBackend, Settings};

use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::SqliteConnection;
use models::*;
use schema::*;

use clap;
use diesel;
use http;
use message;
use opml;

pub fn get_vec(indexes: Option<clap::Values>) -> Option<Vec<i32>> {
    let mut output = Vec::new();
    if let Some(indexes) = indexes {
        for index in indexes {
            let index = i32::from_str(index).unwrap();
            output.push(index);
        }
        Some(output)
    } else {
        None
    }
}

pub fn add(db: &SqliteConnection, name_1: &str, url_1: &str) {
    let feed = NewFeed {
        name: name_1,
        url: url_1,
        paused: false,
        last_seen: 0,
    };
    diesel::insert_into(feeds::dsl::feeds)
        .values(&feed)
        .execute(db)
        .unwrap();
}

pub fn list(db: &SqliteConnection) {
    let results = feeds::dsl::feeds.load::<Feeds>(db).unwrap();
    for feed in results {
        println!(
            "{}: [{}] {} ({})",
            feed.id,
            if feed.paused { " " } else { "*" },
            feed.name,
            feed.url,
        );
    }
}

pub fn pause(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_pause = feeds::dsl::feeds.filter(feeds::dsl::id.eq_any(idxs));
        diesel::update(to_pause)
            .set(feeds::dsl::paused.eq(true))
            .execute(db)
            .unwrap();
    } else {
        diesel::update(feeds::dsl::feeds)
            .set(feeds::dsl::paused.eq(true))
            .execute(db)
            .unwrap();
    }
}

pub fn unpause(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_unpause = feeds::dsl::feeds.filter(feeds::dsl::id.eq_any(idxs));
        diesel::update(to_unpause)
            .set(feeds::dsl::paused.eq(false))
            .execute(db)
            .unwrap();
    } else {
        diesel::update(feeds::dsl::feeds)
            .set(feeds::dsl::paused.eq(false))
            .execute(db)
            .unwrap();
    }
}

pub fn delete(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_delete = feeds::dsl::feeds.filter(feeds::dsl::id.eq_any(idxs));
        diesel::delete(to_delete).execute(db).unwrap();
    } else {
        diesel::delete(feeds::dsl::feeds).execute(db).unwrap();
    }
}

pub fn reset(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_delete = feeds_seen::dsl::feeds_seen.filter(feeds_seen::dsl::parent_id.eq_any(idxs));
        diesel::delete(to_delete).execute(db).unwrap();
    } else {
        diesel::delete(feeds_seen::dsl::feeds_seen)
            .execute(db)
            .unwrap();
    }
}

pub fn opmlimport(db: &SqliteConnection, path: Option<&str>) {
    opml::import(db, path.unwrap());
}

pub fn opmlexport(db: &SqliteConnection, path: Option<&str>) {
    opml::export(db, path.unwrap());
}

// awful hack
enum Lt {
    FileEmailTransport(FileEmailTransport),
    SendmailTransport(SendmailTransport),
}

pub fn run(settings: &Settings, db: &SqliteConnection, no_send: bool) {
    // awful hack since i can not get my trait object to work
    let mut sender = match &settings.mail {
        &MailBackend::File { ref path } => Lt::FileEmailTransport(FileEmailTransport::new(path)),
        &MailBackend::SendMail { ref path } => match path {
            &Some(ref path) => {
                Lt::SendmailTransport(SendmailTransport::new_with_command(path.clone()))
            }
            &None => Lt::SendmailTransport(SendmailTransport::new()),
        },
    };
    let unpaused_feeds_request = feeds::dsl::feeds.filter(feeds::dsl::paused.eq(false));
    let feeds = unpaused_feeds_request.get_results::<Feeds>(db).unwrap();
    for feed in feeds {
        trace!("Starting the treatment of feed {}: {}", feed.name, feed.url);
        trace!("Retrieving feed at {}", feed.url);
        match http::get_feed(&feed.url) {
            Err(err) => {
                error!("{} {}", feed.name, err);
            }
            Ok(data) => {
                trace!("Create mails from feeds");
                match message::Messages::new(&settings, &data) {
                    Err(msg) => {
                        error!("{} {}: {}", feed.name, feed.url, msg);
                    }
                    Ok(messages) => {
                        trace!("Filtering out seen feeds");
                        for (id, message) in messages.vec {
                            let has_been_seen = feeds_seen::dsl::feeds_seen
                                .count()
                                .filter(feeds_seen::dsl::url.eq(&id))
                                .filter(feeds_seen::dsl::parent_id.eq(feed.id));
                            let count: i64 = has_been_seen.get_result(db).unwrap();
                            if !no_send && count == 0 {
                                trace!("Sending mail of feed id '{}'", id);
                                // awful hack
                                match &mut sender {
                                    &mut Lt::FileEmailTransport(ref mut i) => {
                                        match i.send(&message) {
                                            Ok(_) => (),
                                            Err(e) => error!("{}", e),
                                        }
                                    }
                                    &mut Lt::SendmailTransport(ref mut i) => match i.send(&message)
                                    {
                                        Ok(_) => (),
                                        Err(e) => error!("{}", e),
                                    },
                                }
                                let new_feed_seen = NewFeedSeen {
                                    parent_id: feed.id,
                                    url: &id,
                                };
                                trace!("Marking feed id '{}' as seen", id);
                                diesel::insert_into(feeds_seen::dsl::feeds_seen)
                                    .values(&new_feed_seen)
                                    .execute(db)
                                    .unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
