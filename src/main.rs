extern crate atom_syndication;
#[macro_use]
extern crate clap;
extern crate encoding;
#[macro_use]
extern crate error_chain;
extern crate html2text;
extern crate lettre;
extern crate lettre_email;
extern crate reqwest;
extern crate rss;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate xdg;
extern crate xml;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod errors;
mod http;
mod message;
mod opml;
mod settings;
mod schema;
mod models;


use std::str::FromStr;

use lettre::EmailTransport;
use lettre::file::FileEmailTransport;
use lettre::sendmail::SendmailTransport;

use settings::{MailBackend, Settings};

use diesel::prelude::*;
use diesel::Connection;
use diesel::SqliteConnection;
use diesel::ExpressionMethods;
use models::*;
use schema::*;

embed_migrations!("migrations");

fn get_vec(indexes: Option<clap::Values>) -> Option<Vec<i32>> {
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

fn add(db: &SqliteConnection, name_1: &str, url_1: &str) {
        let feed = NewFeed {
            name: name_1,
            url: url_1,
            paused: false,
            last_seen: 0
        };
        diesel::insert_into(feeds::dsl::feeds).values(&feed).execute(db).unwrap();
}

fn list(db: &SqliteConnection) {
    let results = feeds::dsl::feeds.load::<Feeds>(db).unwrap();
    for feed in results {
        println!("{}: [{}] {} ({})",
        feed.id,
        if feed.paused { " " } else { "*" },
        feed.name,
        feed.url,
        );
    }
}

fn pause(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_pause = feeds::dsl::feeds.filter(feeds::dsl::id.eq_any(idxs));
        diesel::update(to_pause).set(feeds::dsl::paused.eq(true)).execute(db).unwrap();
    } else {
        diesel::update(feeds::dsl::feeds).set(feeds::dsl::paused.eq(true)).execute(db).unwrap();
    }
}

fn unpause(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_pause = feeds::dsl::feeds.filter(feeds::dsl::id.eq_any(idxs));
        diesel::update(to_pause).set(feeds::dsl::paused.eq(false)).execute(db).unwrap();
    } else {
        diesel::update(feeds::dsl::feeds).set(feeds::dsl::paused.eq(false)).execute(db).unwrap();
    }
}

fn delete(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_delete = feeds::dsl::feeds.filter(feeds::dsl::id.eq_any(idxs));
        diesel::delete(to_delete).execute(db).unwrap();
    } else {
        diesel::delete(feeds::dsl::feeds).execute(db).unwrap();
    }
}

fn reset(db: &SqliteConnection, indexes: Option<clap::Values>) {
    if let Some(idxs) = get_vec(indexes) {
        let to_delete = feeds_seen::dsl::feeds_seen.filter(feeds_seen::dsl::parent_id.eq_any(idxs));
        diesel::delete(to_delete).execute(db).unwrap();
    } else {
        diesel::delete(feeds_seen::dsl::feeds_seen).execute(db).unwrap();
    }
}

fn opmlimport(db: &SqliteConnection, path: Option<&str>) {
    opml::import(db, path.unwrap());
}

fn opmlexport(db: &SqliteConnection, path: Option<&str>) {
    opml::export(db, path.unwrap());
}

// awful hack
enum Lt {
    FileEmailTransport(FileEmailTransport),
    SendmailTransport(SendmailTransport)
}

fn run(settings: &Settings, db: &SqliteConnection, no_send: bool) {
    // awful hack since i can not get my trait object to work
    let mut sender = match &settings.mail {
        &MailBackend::File{ref path} => Lt::FileEmailTransport(FileEmailTransport::new(path)),
        &MailBackend::SendMail{ref path} => match path {
            &Some(ref path) => Lt::SendmailTransport(SendmailTransport::new_with_command(path.clone())),
            &None => Lt::SendmailTransport(SendmailTransport::new())
        }
    };
    let feeds = feeds::dsl::feeds.load::<Feeds>(db).unwrap();
    for feed in feeds {
        println!("{}", feed.url);
        match http::get_feed(&feed.url) {
            Err(err) => {
                println!("{} {}", feed.name, err);
            }
            Ok(data) => {
                match message::Messages::new(&settings, &data) {
                    Err(msg) => {
                        println!("{} {}: {}", feed.name, feed.url, msg);
                    }
                    Ok(messages) => {
                        for (id, message) in messages.vec {
                            if !no_send && !feeds.has_been_seen(feed.id, &id) {
                                // awful hack
                                match &mut sender {
                                    &mut Lt::FileEmailTransport(ref mut i) => match i.send(&message) {
                                        Ok(_) => (),
                                        Err(e) => eprintln!("{}", e)
                                    },
                                    &mut Lt::SendmailTransport(ref mut i) => match i.send(&message) {
                                        Ok(_) => (),
                                        Err(e) => eprintln!("{}", e)
                                    }
                                }
                            }
                            feeds.see(feed.id, &id);
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let matches = clap_app!(rust2email =>
                            (version: crate_version!())
                            (about: "get RSS and Atom feeds emailed to you")
                            (@setting SubcommandRequiredElseHelp)
                            (@arg config: -c --config +takes_value "path to the configuration file")
                            (@arg data: -d --data +takes_value "path to the data file")
                            (@arg verbose: -v --verbose "increment verbosity")
                            (@subcommand run =>
                             (about: "Fetch feeds and send entry emails")
                             (@arg nosend: -n --nosend "fetch feeds, but don't send email")
                            )
                            (@subcommand add =>
                             (about: "Add a new feed to the database")
                             (@arg name: +required "name of the new feed")
                             (@arg url: +required "location of the new feed")
                            )
                            (@subcommand list =>
                             (about: "List all the feeds in the database")
                            )
                            (@subcommand pause =>
                             (about: "Pause a feed (disable fetching)")
                             (@arg index: +multiple "feed indexes")
                            )
                            (@subcommand unpause =>
                             (about: "Unpause a feed (enable fetching)")
                             (@arg index: +multiple "feed indexes")
                            )
                            (@subcommand delete =>
                             (about: "Remove a feed from the database")
                             (@arg index: +multiple +required "feed indexes")
                            )
                            (@subcommand reset =>
                             (about: "Forget dynamic feed data (e.g. to re-send old entries)")
                             (@arg index: +multiple "feed indexes")
                            )
                            (@subcommand opmlimport =>
                             (about: "Import configuration from OPML.")
                             (@arg path: +required "path for imported OPML")
                            )
                            (@subcommand opmlexport =>
                             (about: "Export configuration from OPML.")
                             (@arg path: +required "path for exported OPML")
                            )
                           )
            .get_matches();

    let settings = Settings::new(matches.value_of("config")).unwrap();

    let xdg_dirs = xdg::BaseDirectories::with_prefix("rust2email").unwrap();
    let data_file = match matches.value_of("data") {
        Some(path) => path.into(),
        None => xdg_dirs.place_data_file("rust2email.db").unwrap(),
    };
    let db = diesel::sqlite::SqliteConnection::establish(data_file.to_str().unwrap()).unwrap();
    embedded_migrations::run(&db);

    match matches.subcommand() {
        //("run", Some(command)) => run(&settings, &mut feeds, command.is_present("nosend")),
        ("add", Some(command)) => {
            add(&db,
                command.value_of("name").unwrap(),
                command.value_of("url").unwrap())
        }
        ("list", Some(_)) => list(&db),
        ("pause", Some(command)) => pause(&db, command.values_of("index")),
        ("unpause", Some(command)) => unpause(&db, command.values_of("index")),
        ("delete", Some(command)) => delete(&db, command.values_of("index")),
        ("reset", Some(command)) => reset(&db, command.values_of("index")),
        ("opmlimport", Some(command)) => opmlimport(&db, command.value_of("path")),
        ("opmlexport", Some(command)) => opmlexport(&db, command.value_of("path")),
        _ => {}
    }
}
