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
extern crate slog;
extern crate sloggers;
extern crate toml;
extern crate xdg;
extern crate xml;
extern crate rusqlite;

mod errors;
//mod feeds;
mod sqlite;
mod http;
mod message;
mod opml;
mod settings;

use std::str::FromStr;

use lettre::EmailTransport;
use lettre::file::FileEmailTransport;
use lettre::sendmail::SendmailTransport;

use settings::{MailBackend, Settings};
use sqlite::Feeds;

use sloggers::Build;
use sloggers::terminal::{TerminalLoggerBuilder, Destination};
use sloggers::types::Severity;

fn get_vec(indexes: Option<clap::Values>) -> Option<Vec<i64>> {
    let mut output = Vec::new();
    if let Some(indexes) = indexes {
        for index in indexes {
            let index = i64::from_str(index).unwrap();
            output.push(index);
        }
        Some(output)
    } else {
        None
    }
}

fn add(feeds: &mut Feeds, name: &str, url: &str) {
        feeds.add_feed(&name.to_string(), &url.to_string());
}

fn list(feeds: &Feeds) {
    let feeds = feeds.get_feeds();
    for feed in feeds {
        println!("{}: [{}] {} ({})",
        feed.id,
        if feed.paused { " " } else { "*" },
        feed.name,
        feed.url,
        );
    }
}

fn pause(feeds: &mut Feeds, indexes: Option<clap::Values>) {
    feeds.pause(get_vec(indexes).as_ref());
}

fn unpause(feeds: &mut Feeds, indexes: Option<clap::Values>) {
    feeds.unpause(get_vec(indexes).as_ref());
}

fn delete(feeds: &mut Feeds, indexes: Option<clap::Values>) {
    feeds.delete(get_vec(indexes).as_ref());
}

fn reset(feeds: &mut Feeds, indexes: Option<clap::Values>) {
    feeds.reset(get_vec(indexes).as_ref());
}

fn opmlimport(mut feeds: &mut Feeds, path: Option<&str>) {
    opml::import(&mut feeds, path.unwrap());
}

fn opmlexport(mut feeds: &mut Feeds, path: Option<&str>) {
    opml::export(&mut feeds, path.unwrap());
}

// awful hack
enum Lt {
    FileEmailTransport(FileEmailTransport),
    SendmailTransport(SendmailTransport)
}

fn run(settings: &Settings, feeds: &mut Feeds, no_send: bool) {
    // awful hack since i can not get my trait object to work
    let mut sender = match &settings.mail {
        &MailBackend::File{ref path} => Lt::FileEmailTransport(FileEmailTransport::new(path)),
        &MailBackend::SendMail{ref path} => match path {
            &Some(ref path) => Lt::SendmailTransport(SendmailTransport::new_with_command(path.clone())),
            &None => Lt::SendmailTransport(SendmailTransport::new())
        }
    };
    let feeds_iter = feeds.get_active_feeds();
    for feed in feeds_iter {
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

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(Severity::Debug);
    builder.destination(Destination::Stderr);
    let logger = builder.build().unwrap();

    let settings = Settings::new(matches.value_of("config")).unwrap();

    let mut feeds = Feeds::new(matches.value_of("data")).unwrap();

    match matches.subcommand() {
        ("run", Some(command)) => run(&settings, &mut feeds, command.is_present("nosend")),
        ("add", Some(command)) => {
            add(&mut feeds,
                command.value_of("name").unwrap(),
                command.value_of("url").unwrap())
        }
        ("list", Some(_)) => list(&mut feeds),
        ("pause", Some(command)) => pause(&mut feeds, command.values_of("index")),
        ("unpause", Some(command)) => unpause(&mut feeds, command.values_of("index")),
        ("delete", Some(command)) => delete(&mut feeds, command.values_of("index")),
        ("reset", Some(command)) => reset(&mut feeds, command.values_of("index")),
        ("opmlimport", Some(command)) => opmlimport(&mut feeds, command.value_of("path")),
        ("opmlexport", Some(command)) => opmlexport(&mut feeds, command.value_of("path")),
        _ => {}
    }
}
