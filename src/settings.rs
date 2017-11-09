use app_dirs::{get_app_root, AppDataType};
use std::path::PathBuf;

use crate::APP_INFO;
use config::{Config, File};

#[derive(Serialize)]
pub struct Feeds {
    pub feeds: Vec<Feed>,
}

impl Feeds {
    pub fn new() -> Feeds {
        Feeds { feeds: Vec::new() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub text: bool,
    pub text_wrap: usize,
    pub from_address: String,
    pub from_display_name: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub mail_backend: Option<MailBackend>,
    pub feeds: Vec<Feed>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MailBackend {
    File { path: String },
    SendMail { path: Option<String> },
}

impl Settings {
    pub fn new(possible_path: &Option<PathBuf>) -> Result<Self, failure::Error> {
        let mut s = Config::new();

        let config_file: PathBuf = match possible_path {
            Some(path) => path.into(),
            None => match get_app_root(AppDataType::UserConfig, &APP_INFO) {
                Ok(path) => path.join("rust2email.toml"),
                Err(err) => panic!(err),
            },
        };

        s.set_default("text", false)?;
        s.set_default("text_wrap", 80)?;
        s.set_default("from_address", "rust2email@does.not.exists.tld")?;
        s.set_default("from_display_name", "<feed_name>")?;
        s.set_default("subject", "<entry_name>")?;
        s.set_default("body", "URL: <entry_url></p>\r\n<entry_body>")?;
        s.set_default::<Vec<String>>("feeds", Vec::new())?;

        s.merge(File::from(config_file))?;

        Ok(s.try_into()?)
    }
}
