use serde_json::json;
use std::error::Error;
use webkit6::{prelude::*, WebView};
use xmltree::{AttributeMap, Element, EmitterConfig, XMLNode};

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
        // Ideally, these namespaced elements are XML-parsed too. But the sources do not contain
        // namespace declaration which makes the parser abort with an error...
        .replace("<g:background", "<img")
        .replace("</g:background", "</img")
        .replace("<g:image", "<img")
        .replace("</g:image", "</img");

    let mut dom = Element::parse(html.as_bytes())?;

    let head: &mut Element = xml_query_mut!(&mut dom, html > head).ok_or("missing <head>")?;
    // Drop the Content-Type meta tag. Gadget files are typically UTF16 encoded, which this tag
    // specifies. We re-encode it to UTF8.
    head.children.retain(|e| match e {
        XMLNode::Element(e) => !xml_query!(e, meta[http_equiv = "Content-Type"]).is_some(),
        _ => true,
    });
    // Insert the polyfill.
    head.children.splice(
        0..0,
        [
            script_node(include_str!("polyfill.js")),
            script_node(format!("window.System.Machine = {};", machine_stats(&sys))),
        ],
    );

    let mut buf = Vec::new();
    dom.write_with_config(
        &mut buf,
        EmitterConfig {
            // Keep the source readable.
            perform_indent: true,
            // Always use verbose </tag> closing elements. Browsers otherwise interpret the
            // <script src/> tags as non-closing, rendering the rest of the file as JS.
            // This does create invalid closing tags such as <img/>, but those are ignored.
            normalize_empty_elements: false,
            ..EmitterConfig::default()
        },
    )?;
    let html = String::from_utf8(buf)?
        // These files render best without a doctype.
        .replace(r#"<?xml version="1.0" encoding="UTF-8"?>"#, "");

    Ok(html.into())
}

fn script_node(src: impl Into<String>) -> XMLNode {
    XMLNode::Element(Element {
        children: vec![XMLNode::Text(src.into())],
        attributes: AttributeMap::from([("type".to_string(), "text/javascript".to_string())]),
        ..Element::new("script")
    })
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
