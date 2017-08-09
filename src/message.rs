use atom_syndication;
use lettre::email::Email;
use lettre::email::EmailBuilder;
use rss;
use settings::Settings;
use html2text;
use errors::*;

pub struct Messages {
    pub vec: Vec<(String, Email)>,
}

impl Messages {

    fn build_message(settings: &Settings,
                     feed_name: &str,
                     entry_name: &str,
                     entry_url: &str,
                     entry_text: &str) -> Email {

            let subject = settings.subject.clone()
                .replace(r"<feed_name>", feed_name)
                .replace(r"<entry_url>", entry_url)
                .replace(r"<entry_name>", entry_name)
                .replace(r"<entry_body>", entry_text);
            let from = settings.from_display_name.clone()
                .replace(r"<feed_name>", feed_name)
                .replace(r"<entry_url>", entry_url)
                .replace(r"<entry_name>", entry_name)
                .replace(r"<entry_body>", entry_text);
            let mut body = settings.body.clone()
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

            email = match settings.text {
                true => email.text(body.as_str()),
                false => email.html(body.as_str()),
            };

            email.build().unwrap()
    }

    fn from_rss(settings: &Settings, channel: &rss::Channel) -> Self {
        let mut messages = Messages { vec: Vec::new() };
        for item in channel.items().iter() {
            let link = item.link().clone().unwrap();
            let text = if let Some(text) = item.content().clone() {
                text
            } else {
                if let Some(text) = item.description().clone() {
                    text
                } else {
                    ""
                }
            };

            let email = Messages::build_message(&settings,
                                      channel.title(),
                                      item.title().unwrap(),
                                      link,
                                      text);

            messages.vec.push((link.to_string(), email));
        }
        messages
    }

    fn from_atom(settings: &Settings, feed: &atom_syndication::Feed) -> Self {
        let mut messages = Messages { vec: Vec::new() };
        for entry in feed.entries().iter() {
            let id = entry.id().clone();
            let text = "";
            let text = match entry.content() {
                Some(content) => match content.value() {
                    Some(value) => value,
                    None => text,
                },
                None => text
            };

            let text = match text {
                "" => match entry.summary() {
                    Some(summary) => summary,
                    _ => text
                },
                &_ => text
            };

            let link = match entry.links().first() {
                Some(link) => link.href(),
                None => "",
            };

            let email = Messages::build_message(&settings,
                                      feed.title(),
                                      entry.title(),
                                      link,
                                      text);

            messages.vec.push((id.to_string(), email));
        }
        messages
    }

    pub fn new(settings: &Settings, data: &str) -> Result<Self> {
        match atom_syndication::Feed::read_from(data.as_bytes()) {
            Ok(feed) => Ok(Messages::from_atom(&settings, &feed)),
            _ => {
                match rss::Channel::read_from(data.as_bytes()) {
                    Ok(channel) => Ok(Messages::from_rss(&settings, &channel)),
                    _ => Err("Could not parse as RSS or Atom".into()),
                }
            }
        }
    }
}
