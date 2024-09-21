use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

const POLYFILL_FILE: &str = "_polyfill.js";

fn decode_ms_string(b: &[u8]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let txt = match &b[0..2] {
        b"\xfe\xff" => {
            let bu16 = b
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>();
            String::from_utf16(&bu16)?
        }
        b"\xff\xfe" => {
            let bu16 = b
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>();
            String::from_utf16(&bu16)?
        }
        _ => String::from_utf8(b.to_vec())?,
    };
    Ok(txt)
}

pub fn inject_polyfill(html: &[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let html = decode_ms_string(html)?
        .replace(
            r#"<meta http-equiv="Content-Type" content="text/html; charset=Unicode" />"#,
            "",
        )
        .replace(
            "<head>",
            &format!(
                r#"<head><script src="{}" type="text/javascript"></script>"#,
                POLYFILL_FILE
            ),
        )
        .replace("<g:background", "<img")
        .replace("</g:background", "</img")
        .replace("<g:image", "<img")
        .replace("</g:image", "</img");

    Ok(html.into())
}

pub fn write_polyfill(dir: impl AsRef<Path>) -> io::Result<()> {
    let dir = dir.as_ref();

    let mut f = File::create(dir.join(POLYFILL_FILE))?;
    f.write_all(include_bytes!("polyfill.js"))
}
