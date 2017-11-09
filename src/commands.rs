use crate::database::Database;
use crate::http;
use crate::message;
use crate::settings::{MailBackend, Settings};
use failure::Error;
use lettre::Transport;
use lettre::{FileTransport, SendmailTransport};
use log::{trace, warn};
use std::collections::HashSet;
use std::path::Path;

pub fn opml_to_toml(path: &Path) -> Result<(), Error> {
    let feeds = crate::opml::opml_to_model(&path)?;
    let toml = toml::to_string(&feeds)?;
    print!("{}", toml);
    Ok(())
}

pub fn feeds_to_opml(path: &Path, settings: &Settings) -> Result<(), Error> {
    crate::opml::export_to_opml(&path, &settings)?;
    Ok(())
}

// awful hack
enum Lt {
    FileTransport(FileTransport),
    SendmailTransport(SendmailTransport),
}

pub fn run_feeds(no_send: bool, settings: &Settings, database: &mut Database) -> Result<(), Error> {
    // awful hack since i can not get my trait object to work
    let mut sender = match &settings.mail_backend {
        Some(MailBackend::File { ref path }) => Lt::FileTransport(FileTransport::new(path)),
        Some(MailBackend::SendMail { ref path }) => match path {
            Some(ref path) => {
                Lt::SendmailTransport(SendmailTransport::new_with_command(path.clone()))
            }
            None => Lt::SendmailTransport(SendmailTransport::new()),
        },
        None => bail!("A mail backend is required. See the `mail_backend` options"),
    };

    for feed in &settings.feeds {
        trace!("Starting the treatment of feed `{}`", feed.name);
        trace!("Retrieving feed at `{}`", feed.url);

        let mut new_seen_feeds = HashSet::new();
        let opt_seen_feeds = database.data.seens.get(&feed.url);

        let http_data = match http::get_feed(&feed.url) {
            Err(err) => {
                warn!("{} {}", feed.name, err);
                continue;
            }
            Ok(data) => data,
        };

        trace!("Create mails from feeds");

        let messages = match message::Messages::new(&settings, &http_data) {
            Err(msg) => {
                warn!("{} {}: {}", feed.name, feed.url, msg);
                continue;
            }
            Ok(messages) => messages,
        };

        for feed_mail in messages.vec {
            new_seen_feeds.insert(feed_mail.id.to_string());

            if no_send {
                continue;
            }

            let mut feed_seen = false;

            if let Some(seen_feeds) = opt_seen_feeds {
                if seen_feeds.contains(&feed_mail.id) {
                    feed_seen = true;
                }
            }

            if !feed_seen {
                trace!("Sending mail of feed id '{}'", feed_mail.id);
                // awful hack
                match sender {
                    Lt::FileTransport(ref mut i) => {
                        i.send(feed_mail.email.into())?;
                    }
                    Lt::SendmailTransport(ref mut i) => {
                        i.send(feed_mail.email.into())?;
                    }
                }
            } else {
                trace!("feed `{}` already seen", feed_mail.id);
            }
        }
        if !new_seen_feeds.is_empty() {
            database.update(&feed.url, new_seen_feeds)?;
        }
    }
    database.unlock()?;
    Ok(())
}
