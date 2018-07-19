use failure::Error;
use std::fs::File;
use std::io::Read;
use toml;

use app_dirs::{get_app_root, AppDataType};
use std::path::PathBuf;
use APP_INFO;

#[derive(Debug, Deserialize)]
struct ConfigFileMailFile {
    path: String,
}

#[derive(Debug, Deserialize)]
struct ConfigFileMailSendMail {
    path: String,
}

#[derive(Debug, Deserialize)]
struct ConfigFileSettings {
    verbose: Option<bool>,
    text: Option<bool>,
    text_wrap: Option<usize>,
    from_address: Option<String>,
    from_display_name: Option<String>,
    to: String,
    subject: Option<String>,
    body: Option<String>,
    mail_backend: String,
    mail_file: Option<ConfigFileMailFile>,
    mail_sendmail: Option<ConfigFileMailSendMail>,
}

pub enum MailBackend {
    File { path: String },
    SendMail { path: Option<String> },
}

pub struct Settings {
    pub verbose: bool,
    pub text: bool,
    pub text_wrap: usize,
    pub from_address: String,
    pub from_display_name: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub mail: MailBackend,
}

impl Settings {
    pub fn new(path: Option<&str>) -> Result<Self, Error> {
        let config_file: PathBuf = match path {
            Some(path) => path.into(),
            None => match get_app_root(AppDataType::UserConfig, &APP_INFO) {
                Ok(path) => path.join("rust2email.toml"),
                Err(err) => panic!(err),
            },
        };

        let mut f = File::open(config_file).unwrap();
        let mut data = String::new();
        f.read_to_string(&mut data).unwrap();

        let file_config: ConfigFileSettings = toml::from_str(data.as_str()).unwrap();

        let mail = match file_config.mail_backend.as_str() {
            "file" => match file_config.mail_file {
                Some(file) => MailBackend::File { path: file.path },
                None => bail!("file backend selected but no path given"),
            },
            "sendmail" => match file_config.mail_sendmail {
                Some(sendmail) => MailBackend::SendMail {
                    path: Some(sendmail.path),
                },
                None => MailBackend::SendMail { path: None },
            },
            _ => bail!("wrong or no mail backend selected"),
        };

        Ok(Settings {
            verbose: match file_config.verbose {
                Some(verbose) => verbose,
                None => false,
            },
            text: match file_config.text {
                Some(text) => text,
                None => false,
            },
            text_wrap: match file_config.text_wrap {
                Some(wrap) => wrap,
                None => 80,
            },
            from_address: match file_config.from_address {
                Some(from) => from,
                None => "user@rust2email.invalid".into(),
            },
            from_display_name: match file_config.from_display_name {
                Some(from) => from,
                None => "<feed_name>".into(),
            },
            to: file_config.to,
            subject: match file_config.subject {
                Some(subject) => subject,
                None => "<entry_name>".into(),
            },
            body: match file_config.body {
                Some(body) => body,
                None => "<p>URL: <entry_url></p>\r\n<entry_body>".into(),
            },
            mail: mail,
        })
    }
}
