extern crate atom_syndication;
#[macro_use]
extern crate clap;
extern crate encoding;
extern crate html2text;
extern crate lettre;
extern crate lettre_email;
extern crate reqwest;
extern crate rss;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate app_dirs;
extern crate toml;
extern crate xml;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate failure;
extern crate stderrlog;
#[macro_use]
extern crate log;

mod commands;
mod http;
mod message;
mod models;
mod opml;
mod schema;
mod settings;

use settings::Settings;

use diesel::prelude::*;

use app_dirs::{app_root, AppDataType, AppInfo};
use std::path::PathBuf;

const APP_INFO: AppInfo = AppInfo {
    name: "rust2email",
    author: "nurelin",
};

embed_migrations!("migrations");

fn main() {
    let matches = clap_app!(rust2email =>
                            (version: crate_version!())
                            (about: "get RSS and Atom feeds emailed to you")
                            (@setting SubcommandRequiredElseHelp)
                            (@arg config: -c --config +takes_value "path to the configuration file")
                            (@arg data: -d --data +takes_value "path to the data file")
                            (@arg verbose: -v ... "increment verbosity")
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
                           ).get_matches();

    stderrlog::new()
        .module(module_path!())
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .unwrap();

    let settings = Settings::new(matches.value_of("config")).unwrap();

    let data_file: PathBuf = match matches.value_of("data") {
        Some(path) => path.into(),
        None => match app_root(AppDataType::UserData, &APP_INFO) {
            Ok(path) => path.join("rust2email.db"),
            Err(err) => panic!(err),
        },
    };
    let db = diesel::sqlite::SqliteConnection::establish(data_file.to_str().unwrap()).unwrap();
    embedded_migrations::run(&db).unwrap();

    match matches.subcommand() {
        ("run", Some(command)) => commands::run(&settings, &db, command.is_present("nosend")),
        ("add", Some(command)) => commands::add(
            &db,
            command.value_of("name").unwrap(),
            command.value_of("url").unwrap(),
        ),
        ("list", Some(_)) => commands::list(&db),
        ("pause", Some(command)) => commands::pause(&db, command.values_of("index")),
        ("unpause", Some(command)) => commands::unpause(&db, command.values_of("index")),
        ("delete", Some(command)) => commands::delete(&db, command.values_of("index")),
        ("reset", Some(command)) => commands::reset(&db, command.values_of("index")),
        ("opmlimport", Some(command)) => commands::opmlimport(&db, command.value_of("path")),
        ("opmlexport", Some(command)) => commands::opmlexport(&db, command.value_of("path")),
        _ => {}
    }
}
