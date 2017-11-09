use crate::settings::{Feed, Feeds, Settings};
use failure::Error;
use log::warn;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::Path;
use xml::{reader, writer};

pub fn opml_to_model(path: &Path) -> Result<Feeds, Error> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let mut output = Feeds::new();

    let parser = reader::EventReader::new(file);
    for e in parser {
        match e {
            Ok(reader::XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                if name.local_name == "outline" {
                    let mut name: Option<String> = None;
                    let mut url: Option<String> = None;
                    for attribute in attributes {
                        match attribute.name.local_name.as_str() {
                            "text" => name = Some(attribute.value),
                            "xmlUrl" => url = Some(attribute.value),
                            _ => {}
                        }
                    }
                    match name {
                        Some(name) => match url {
                            Some(url) => {
                                output.feeds.push(Feed { url, name });
                            }
                            None => warn!("Feed with name {} has no url", name),
                        },
                        None => {
                            if let Some(url) = url {
                                warn!("Feed with url {} has no name", url)
                            } else {
                                panic!()
                            }
                        }
                    }
                }
            }
            Err(e) => unimplemented!(),
            _ => {}
        }
    }
    Ok(output)
}

pub fn export_to_opml(path: &Path, settings: &Settings) -> Result<(), Error> {
    let file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let mut writer = writer::EmitterConfig::new()
        .perform_indent(true)
        .create_writer(file);

    writer.write(writer::XmlEvent::start_element("opml").attr("version", "2.0"))?;
    writer.write(writer::XmlEvent::start_element("head"))?;
    writer.write(writer::XmlEvent::start_element("title"))?;
    writer.write(writer::XmlEvent::characters("rust2email OPML export"))?;
    writer.write(writer::XmlEvent::end_element())?;
    writer.write(writer::XmlEvent::end_element())?;
    writer.write(writer::XmlEvent::start_element("body"))?;

    for feed in &settings.feeds {
        writer.write(
            writer::XmlEvent::start_element("outline")
                .attr("title", feed.name.as_str())
                .attr("text", feed.name.as_str())
                .attr("xmlUrl", feed.url.as_str()),
        )?;
        writer.write(writer::XmlEvent::end_element())?;
    }

    writer.write(writer::XmlEvent::end_element())?;
    writer.write(writer::XmlEvent::end_element())?;
    Ok(())
}
