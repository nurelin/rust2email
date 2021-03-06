use xml::{reader, writer};
use feeds;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::collections::HashMap;
use errors::*;

fn get_map(path: &str) -> Result<HashMap<String, String>> {
    let mut hashmap = HashMap::new();
    let file = File::open(path).unwrap();
    let file = BufReader::new(file);

    let parser = reader::EventReader::new(file);
    for e in parser {
        match e {
            Ok(reader::XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name == "outline" {
                    let mut title = String::new();
                    let mut url = String::new();
                    for attribute in attributes {
                        match attribute.name.local_name.as_str() {
                            "text" => title = attribute.value,
                            "xmlUrl" => url = attribute.value,
                            _ => {}
                        }
                    }
                    hashmap.insert(title, url);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    Ok(hashmap)
}

pub fn import(feeds: &mut feeds::Feeds, path: &str) {
    let mut hashmap = get_map(path).unwrap();

    for ref feed in &feeds.feeds {
        if hashmap.contains_key(feed.name.as_str()) {
            hashmap.remove(feed.name.as_str());
        }
    }

    for (name, url) in hashmap {
        feeds.push(name.as_str(), url.as_str());
    }
}

pub fn export(feeds: &mut feeds::Feeds, path: &str) {
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

    for feed in &feeds.feeds {
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
