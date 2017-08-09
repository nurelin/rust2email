// This file is a modified copy of
// https://github.com/lettre/lettre/blob/master/lettre/src/sendmail/mod.rs
// until sendmail is officially available in lettre

use lettre::transport::EmailTransport;
use lettre::email::SendableEmail;
use sendmail::error::SendmailResult;
use std::io::prelude::*;
use std::process::{Command, Stdio};

pub mod error {
	//! Error and result type for sendmail transport

	use self::Error::*;
	use std::error::Error as StdError;
	use std::fmt::{self, Display, Formatter};
	use std::io;

	/// An enum of all error kinds.
#[derive(Debug)]
	pub enum Error {
		/// Internal client error
		Client(&'static str),
		/// IO error
		Io(io::Error),
	}

		impl Display for Error {
			fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
				fmt.write_str(self.description())
			}
		}

		impl StdError for Error {
			fn description(&self) -> &str {
				match *self {
					Client(err) => err,
					Io(ref err) => err.description(),
				}
			}

			fn cause(&self) -> Option<&StdError> {
				match *self {
					Io(ref err) => Some(&*err as &StdError),
					_ => None,
				}
			}
		}

		impl From<io::Error> for Error {
			fn from(err: io::Error) -> Error {
				Io(err)
			}
		}

		impl From<&'static str> for Error {
			fn from(string: &'static str) -> Error {
				Client(string)
			}
		}

		/// sendmail result type
		pub type SendmailResult = Result<(), Error>;
}

/// Sends an email using the `sendmail` command
#[derive(Debug, Default)]
pub struct SendmailTransport {
    command: String,
}

impl SendmailTransport {
    /// Creates a new transport with the default `/usr/sbin/sendmail` command
    pub fn new() -> SendmailTransport {
        SendmailTransport { command: "/usr/sbin/sendmail".to_string() }
    }

    /// Creates a new transport to the given sendmail command
    pub fn new_with_command<S: Into<String>>(command: S) -> SendmailTransport {
        SendmailTransport { command: command.into() }
    }
}

impl EmailTransport<SendmailResult> for SendmailTransport {
    fn send<U: SendableEmail>(&mut self, email: U) -> SendmailResult {
        // Spawn the sendmail command
        let to_addresses: Vec<String> = email.to_addresses().iter().map(|x| x.to_string()).collect();
        let mut process = Command::new(&self.command)
            .args(
                &[
                    "-i",
                    "-f",
                    &email.from_address().to_string(),
                    &to_addresses.join(" "),
                ],
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let message_content = email.message().clone();

        match process.stdin.as_mut().unwrap().write_all(
            message_content.as_bytes(),
        ) {
            Ok(_) => (),
            Err(error) => return Err(From::from(error)),
        }

        //info!("Wrote message to stdin");

        if let Ok(output) = process.wait_with_output() {
            if output.status.success() {
                Ok(())
            } else {
                Err(From::from("The message could not be sent"))
            }
        } else {
            Err(From::from("The sendmail process stopped"))
        }
    }

    fn close(&mut self) {
    }
}
