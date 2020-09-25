use reqwest;
use std::io;

error_chain! {
    foreign_links {
        ReqError(reqwest::Error);
        Io(io::Error);
   }
}
