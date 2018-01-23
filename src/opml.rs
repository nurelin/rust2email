use xml::{reader, writer};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use diesel;
use diesel::prelude::*;
use diesel::SqliteConnection;
use models::*;
use schema::feeds;

pub fn import(db: &SqliteConnection, path: &str) {
    let file = File::open(path).unwrap();
    let file = BufReader::new(file);

    let parser = reader::EventReader::new(file);
    for e in parser {
        match e {
            Ok(reader::XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name == "outline" {
                    let mut name = String::new();
                    let mut url = String::new();
                    for attribute in attributes {
                        match attribute.name.local_name.as_str() {
                            "text" => name = attribute.value,
                            "xmlUrl" => url = attribute.value,
                            _ => {}
                        }
                    }
                    let new_feed = NewFeed {
                        name: &name,
                        url: &url,
                        paused: false,
                        last_seen: 0,
                    };
                    diesel::insert_into(feeds::dsl::feeds)
                        .values(&new_feed)
                        .execute(db)
                        .unwrap();
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

pub fn export(db: &SqliteConnection, path: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    let mut writer = writer::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(file);

    writer
        .write(writer::XmlEvent::start_element("opml").attr("version", "2.0"))
        .unwrap();
    writer
        .write(writer::XmlEvent::start_element("head"))
        .unwrap();
    writer
        .write(writer::XmlEvent::start_element("title"))
        .unwrap();
    writer
        .write(writer::XmlEvent::characters("rust2email OPML export"))
        .unwrap();
    writer.write(writer::XmlEvent::end_element()).unwrap();
    writer.write(writer::XmlEvent::end_element()).unwrap();
    writer
        .write(writer::XmlEvent::start_element("body"))
        .unwrap();

    let feeds = feeds::dsl::feeds.load::<Feeds>(db).unwrap();
    for feed in &feeds {
        writer
            .write(writer::XmlEvent::start_element("outline")
                       .attr("title", feed.name.as_str())
                       .attr("text", feed.name.as_str())
                       .attr("xmlUrl", feed.url.as_str()))
            .unwrap();
        writer.write(writer::XmlEvent::end_element()).unwrap();
    }

    writer.write(writer::XmlEvent::end_element()).unwrap();
    writer.write(writer::XmlEvent::end_element()).unwrap();

}
