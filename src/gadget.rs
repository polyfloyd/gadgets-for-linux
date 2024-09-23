use crate::webhack;
use roxmltree as xml;
use std::error::Error;
use std::fmt;
use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};
use std::path::Path;
use zip::result::ZipError;
use zip::ZipArchive;

#[derive(Debug)]
pub struct Gadget {
    ar: ZipArchive<File>,

    name: String,
    author: Option<String>,
    copyright: Option<String>,

    entrypoint: String,
}

impl Gadget {
    pub fn from_file(filename: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let mut ar = ZipArchive::new(File::open(filename)?)?;

        let manifest_str = {
            let mut manifest_file = match ar.by_name("en-US/gadget.xml") {
                Ok(v) => v,
                Err(ZipError::FileNotFound) => return Err("gadget manifest not found".into()),
                Err(err) => return Err(err.into()),
            };
            let mut manifest_str = String::with_capacity(manifest_file.size() as usize);
            manifest_file.read_to_string(&mut manifest_str)?;
            manifest_str
        };

        let manifest = xml::Document::parse(&manifest_str)?;

        let name = query_xml(manifest.root(), ["gadget", "name"])
            .and_then(|n| text_content(n))
            .ok_or_else(|| "no gadget.name node")?;
        let author = query_xml(manifest.root(), ["gadget", "author"])
            .and_then(|n| n.attribute("name"))
            .map(str::to_string);
        let copyright =
            query_xml(manifest.root(), ["gadget", "copyright"]).and_then(|n| text_content(n));
        let entrypoint = query_xml(manifest.root(), ["gadget", "hosts", "host", "base"])
            .and_then(|n| n.attribute("src"))
            .map(str::to_string)
            .ok_or_else(|| "no gadget html entrypoint node")?;

        Ok(Self {
            ar,
            name,
            author,
            copyright,
            entrypoint,
        })
    }

    pub fn unpack_to(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();

        for file_index in 0.. {
            let mut f = match self.ar.by_index(file_index) {
                Ok(v) => v,
                Err(ZipError::FileNotFound) => break,
                Err(err) => Err(err)?,
            };
            if f.is_dir() {
                continue;
            }

            let fname = Path::new(f.name());
            let is_entrypoint = fname == Path::new("en-US").join(&self.entrypoint);
            let oname = path.join(if is_entrypoint {
                Path::new("index.html")
            } else if let Ok(o) = fname.strip_prefix("en-US/") {
                o
            } else {
                fname
            });

            create_dir_all(oname.parent().unwrap())?;
            let mut of = File::create(oname)?;

            if is_entrypoint {
                let mut html = Vec::with_capacity(f.size() as usize);
                f.read_to_end(&mut html)?;
                html = webhack::inject_polyfill(&html)
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                of.write_all(&html)?;
            } else {
                io::copy(&mut f, &mut of)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Gadget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.name)?;
        if let Some(author) = &self.author {
            write!(f, ", {}", author)?;
            if let Some(copyright) = &self.copyright {
                write!(f, " {}", copyright)?;
            }
        }
        Ok(())
    }
}

fn query_xml<'a>(
    mut root: xml::Node<'a, 'a>,
    path: impl IntoIterator<Item = &'static str>,
) -> Option<xml::Node<'a, 'a>> {
    for p in path.into_iter() {
        root = root.children().filter(|n| n.has_tag_name(p)).next()?;
    }
    Some(root)
}

fn text_content(node: xml::Node) -> Option<String> {
    let texts: Vec<_> = node
        .children()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect();
    if texts.is_empty() {
        None
    } else {
        Some(texts.join(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gadget_from_file() {
        let f = "./testdata/cpu.gadget";

        let gadget = Gadget::from_file(f).unwrap();
        assert_eq!(gadget.name, "CPU Meter");
        assert_eq!(gadget.entrypoint, "cpu.html");
    }
}
