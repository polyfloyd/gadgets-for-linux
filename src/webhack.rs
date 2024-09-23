use serde_json::json;
use std::error::Error;
use webkit6::{prelude::*, WebView};

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
            &format!(r#"<head><script>{}</script>"#, include_str!("polyfill.js")),
        )
        .replace("<g:background", "<img")
        .replace("</g:background", "</img")
        .replace("<g:image", "<img")
        .replace("</g:image", "</img");

    Ok(html.into())
}

pub async fn update_machine_stats(web_view: &WebView, sys: &sysinfo::System) {
    let machine = json!({
        "CPUs": sys.cpus().iter()
            .map(|cpu| json!({"usagePercentage": cpu.cpu_usage()}))
            .collect::<Vec<_>>(),
        "totalMemory": sys.total_memory(),
        "availableMemory": sys.available_memory(),
    });

    let js = format!("window.System.Machine = {}", machine);
    web_view
        .evaluate_javascript_future(&js, None, None)
        .await
        .unwrap();
}
