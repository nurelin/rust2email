use encoding::codec::utf_8::UTF8Encoding;
use encoding::types::DecoderTrap;
use encoding::Encoding;
use failure::Error;
use reqwest;
use std::io::Read;

pub fn get_feed(url: &str) -> Result<String, Error> {
    match reqwest::get(url) {
        Err(err) => Err(err.into()),
        Ok(resp) => match resp.error_for_status() {
            Err(err) => Err(err.into()),
            Ok(resp) => {
                let bytes: Vec<u8> = resp.bytes().map(|res| res.unwrap()).collect();
                match UTF8Encoding.decode(bytes.as_slice(), DecoderTrap::Replace) {
                    Ok(string) => Ok(string),
                    Err(err) => Err(format_err!("Cannot decode as UTF8")),
                }
            }
        },
    }
}
