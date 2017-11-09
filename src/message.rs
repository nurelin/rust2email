use crate::settings::Settings;
use atom_syndication;
use failure::Error;
use html2text;
use lettre_email::Email;
use lettre_email::EmailBuilder;
use rss;

pub struct MailFeed {
    pub id: String,
    pub email: Email,
}

pub struct Messages {
    pub vec: Vec<MailFeed>,
}

impl Messages {
    fn build_message(
        settings: &Settings,
        feed_name: &str,
        entry_name: &str,
        entry_url: &str,
        entry_text: &str,
    ) -> Result<Email, Error> {
        let subject = settings
            .subject
            .clone()
            .replace(r"<feed_name>", feed_name)
            .replace(r"<entry_url>", entry_url)
            .replace(r"<entry_name>", entry_name)
            .replace(r"<entry_body>", entry_text);
        let from = settings
            .from_display_name
            .clone()
            .replace(r"<feed_name>", feed_name)
            .replace(r"<entry_url>", entry_url)
            .replace(r"<entry_name>", entry_name)
            .replace(r"<entry_body>", entry_text);
        let mut body = settings
            .body
            .clone()
            .replace(r"<feed_name>", feed_name)
            .replace(r"<entry_url>", entry_url)
            .replace(r"<entry_name>", entry_name)
            .replace(r"<entry_body>", entry_text);

        if settings.text {
            body = html2text::from_read(body.as_bytes(), settings.text_wrap);
        }
        let mut email = EmailBuilder::new()
            .to(settings.to.as_str())
            .from((settings.from_address.as_str(), from.as_str()))
            .subject(subject.as_str());

        email = if settings.text {
            email.text(body.as_str())
        } else {
            email.html(body.as_str())
        };

        Ok(email.build()?)
    }

    fn from_rss(settings: &Settings, channel: &rss::Channel) -> Result<Self, Error> {
        let mut messages = Messages { vec: Vec::new() };
        for item in channel.items().iter() {
            let link = item.link().unwrap_or("");
            let text = if let Some(text) = item.content() {
                text
            } else if let Some(text) = item.description() {
                text
            } else {
                ""
            };

            let email = Messages::build_message(
                &settings,
                channel.title(),
                item.title().unwrap_or("no_title"),
                link,
                text,
            )?;

            messages.vec.push(MailFeed {
                id: link.to_string(),
                email,
            });
        }
        Ok(messages)
    }

    fn from_atom(settings: &Settings, feed: &atom_syndication::Feed) -> Result<Self, Error> {
        let mut messages = Messages { vec: Vec::new() };
        for entry in feed.entries().iter() {
            let id: String = entry.id().to_string();
            let text = "";
            let text = match entry.content() {
                Some(content) => match content.value() {
                    Some(value) => value,
                    None => text,
                },
                None => text,
            };

            let text = match text {
                "" => match entry.summary() {
                    Some(summary) => summary,
                    _ => text,
                },
                &_ => text,
            };

            let mut link = "";
            for it in entry.links() {
                if link == "" {
                    link = it.href();
                }
                if it.rel() == "alternate" {
                    link = it.href();
                }
            }

            let email =
                Messages::build_message(&settings, feed.title(), entry.title(), link, text)?;

            messages.vec.push(MailFeed {
                id: id.to_string(),
                email,
            });
        }
        Ok(messages)
    }

    pub fn new(settings: &Settings, data: &str) -> Result<Self, Error> {
        match atom_syndication::Feed::read_from(data.as_bytes()) {
            Ok(feed) => Messages::from_atom(&settings, &feed),
            _ => match rss::Channel::read_from(data.as_bytes()) {
                Ok(channel) => Messages::from_rss(&settings, &channel),
                _ => Err(format_err!("Could not parse as RSS or Atom")),
            },
        }
    }
}
