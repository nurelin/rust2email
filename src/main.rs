#![recursion_limit = "1024"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

mod commands;
mod database;
mod http;
mod message;
mod opml;
mod settings;

use crate::database::Database;
use crate::settings::Settings;

use app_dirs::AppInfo;
use std::path::PathBuf;

use failure::Error;
use structopt::StructOpt;

const APP_INFO: AppInfo = AppInfo {
    name: env!("CARGO_PKG_NAME"),
    author: env!("CARGO_PKG_AUTHORS"),
};

#[derive(StructOpt)]
#[structopt(name = "rust2email")]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
    /// Path to config File
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config: Option<PathBuf>,
    /// Optional path to data file
    #[structopt(short = "d", long = "data", parse(from_os_str))]
    data: Option<PathBuf>,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Feeds operations
    #[structopt(name = "feed")]
    Feed {
        /// Only fill the read feeds database
        #[structopt(long = "no-send")]
        no_send: bool,
    },
    /// Output OPML file to toml configuration on stdout
    #[structopt(name = "opml2toml")]
    OPML2TOML {
        /// Path to OPML file
        #[structopt(parse(from_os_str))]
        opml: PathBuf,
    },
    /// Output configuration file feeds to OPML file
    #[structopt(name = "export2opml")]
    ExportOPML {
        /// Path to OPML file
        #[structopt(parse(from_os_str))]
        output: PathBuf,
    },
}

fn run(opt: Opt) -> Result<(), Error> {
    match opt.cmd {
        Command::OPML2TOML { opml } => crate::commands::opml_to_toml(&opml),
        Command::ExportOPML { output } => {
            let settings = Settings::new(&opt.config)?;
            crate::commands::feeds_to_opml(&output, &settings)
        }
        Command::Feed { no_send } => {
            let settings = Settings::new(&opt.config)?;
            let mut data = Database::new(&opt.data)?;
            crate::commands::run_feeds(no_send, &settings, &mut data)
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log::LevelFilter::Error)
        .level_for(
            "rust2email",
            match opt.verbose {
                0 => log::LevelFilter::Info,
                1 => log::LevelFilter::Debug,
                _ => log::LevelFilter::Trace,
            },
        )
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    if let Err(err) = run(opt) {
        eprintln!("error: {}", err);

        //for e in err.iter().skip(1) {
        //    eprintln!("caused by: {}", e);
        //}

        std::process::exit(1);
    }
}
