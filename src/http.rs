use crate::errors::*;
use encoding::codec::utf_8::UTF8Encoding;
use encoding::types::DecoderTrap;
use encoding::Encoding;
use futures::executor::block_on;
use reqwest;

pub fn get_feed(url: &str) -> Result<String> {
    block_on(get_feed_async(url))
}

async fn get_feed_async(url: &str) -> Result<String> {
    match reqwest::get(url).await {
        Err(err) => Err(err.into()),
        Ok(resp) => match resp.error_for_status() {
            Err(err) => Err(err.into()),
            Ok(resp) => {
                let bytes: Vec<u8> = resp
                    .bytes()
                    .await
                    .map(|bytes| Vec::from(bytes.as_ref()))
                    .unwrap();
                match UTF8Encoding.decode(bytes.as_slice(), DecoderTrap::Replace) {
                    Ok(string) => Ok(string),
                    Err(err) => Err(err.to_string().into()),
                }
            }
        },
    }
}
