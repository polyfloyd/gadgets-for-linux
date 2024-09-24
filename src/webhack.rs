use serde_json::json;
use std::error::Error;
use webkit6::{prelude::*, WebView};

fn decode_ms_string(b: &[u8]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let txt = match &b[0..2] {
        b"\xfe\xff" => {
            let bu16 = b
                .chunks_exact(2)
                .skip(1) // BOM
                .map(|c| u16::from_be_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>();
            String::from_utf16(&bu16)?
        }
        b"\xff\xfe" => {
            let bu16 = b
                .chunks_exact(2)
                .skip(1) // BOM
                .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
                .collect::<Vec<_>>();
            String::from_utf16(&bu16)?
        }
        _ => String::from_utf8(b.to_vec())?,
    };
    Ok(txt)
}

pub fn inject_polyfill(html: &[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let sys = sysinfo::System::new_all();

    let html = decode_ms_string(html)?
        .replace(
            r#"<meta http-equiv="Content-Type" content="text/html; charset=Unicode" />"#,
            "",
        )
        .replace(
            "<head>",
            &format!(
                r#"<head>
                <script>
                    {}
                    window.System.Machine = {};
                </script>
            "#,
                include_str!("polyfill.js"),
                machine_stats(&sys)
            ),
        )
        .replace("<g:background", "<img")
        .replace("</g:background", "</img")
        .replace("<g:image", "<img")
        .replace("</g:image", "</img");

    Ok(html.into())
}

fn machine_stats(sys: &sysinfo::System) -> serde_json::Value {
    json!({
        "CPUs": sys.cpus().iter()
            .map(|cpu| json!({"usagePercentage": cpu.cpu_usage()}))
            .collect::<Vec<_>>(),
        "totalMemory": sys.total_memory() / 1_000_000,
        "availableMemory": sys.available_memory() / 1_000_000,
    })
}

pub async fn update_machine_stats(web_view: &WebView, sys: &sysinfo::System) {
    let js = format!("window.System.Machine = {}", machine_stats(sys));
    web_view
        .evaluate_javascript_future(&js, None, None)
        .await
        .unwrap();
}
