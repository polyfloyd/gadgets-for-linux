use crate::webhack;
use std::error::Error;
use std::fmt;
use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use xmltree::Element;
use zip::result::ZipError;
use zip::{read::ZipFile, ZipArchive};

#[derive(Debug)]
pub struct Gadget {
    ar: ZipArchive<File>,

    name: String,
    author: Option<String>,
    copyright: Option<String>,

    entrypoint: PathBuf,
}

impl Gadget {
    pub fn from_file(filename: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let mut ar = ZipArchive::new(File::open(filename)?)?;

        let manifest_str = {
            let mut manifest_file = try_file_by_name(&mut ar, ["gadget.xml", "en-US/gadget.xml"])
                .ok_or("gadget manifest not found")?;
            let mut manifest_str = String::with_capacity(manifest_file.size() as usize);
            manifest_file.read_to_string(&mut manifest_str)?;
            manifest_str
        };

        let manifest = Element::parse(manifest_str.as_bytes())?;

        let name = xml_query!(&manifest, gadget > name)
            .and_then(|n| n.get_text())
            .map(String::from)
            .ok_or_else(|| "no gadget.name node")?;

        let author = xml_query!(&manifest, gadget > author)
            .and_then(|n| n.attributes.get("name"))
            .cloned();
        let copyright = xml_query!(&manifest, gadget > copyright)
            .and_then(|n| n.get_text())
            .map(String::from);
        let entrypoint = xml_query!(&manifest, gadget > hosts > host > base[type~="HTML"])
            .and_then(|n| n.attributes.get("src"))
            .cloned()
            .ok_or_else(|| "no gadget html entrypoint node")?;

        Ok(Self {
            ar,
            name,
            author,
            copyright,
            entrypoint: PathBuf::from(entrypoint),
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

            let fname = f
                .enclosed_name()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid file name"))?;
            // TODO: check for other lang-pairs?
            let fname = fname.strip_prefix("en-US/").unwrap_or(&fname);

            let is_entrypoint = fname == &self.entrypoint;
            let oname = path.join(if is_entrypoint {
                Path::new("index.html")
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

fn try_file_by_name(
    ar: &mut ZipArchive<impl io::Read + io::Seek>,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Option<ZipFile> {
    let i = paths
        .into_iter()
        .filter_map(|p| ar.index_for_path(p))
        .next()?;
    ar.by_index(i).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gadget_from_file() {
        let f = "./testdata/cpu.gadget";

        let gadget = Gadget::from_file(f).unwrap();
        assert_eq!(gadget.name, "CPU Meter");
        assert_eq!(gadget.entrypoint, Path::new("cpu.html"));
    }
}
