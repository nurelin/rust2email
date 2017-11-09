use reqwest;
use rusqlite;
use std::io;

error_chain!{
    foreign_links {
        ReqError(reqwest::Error);
        Io(io::Error);
        Rusqlite(rusqlite::Error);
   }
}
