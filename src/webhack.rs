use html5ever::tree_builder::TreeSink;
use html5ever::{driver::parse_document as parse_html, tendril::TendrilSink};
use markup5ever_rcdom::{Handle, RcDom, SerializableHandle};
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
        // Ideally, these namespaced elements are XML-parsed too. But the sources do not contain
        // namespace declaration which makes the parser abort with an error...
        .replace("<g:background", "<img")
        .replace("</g:background", "</img")
        .replace("<g:image", "<img")
        .replace("</g:image", "</img");

    let parser = parse_html(RcDom::default(), Default::default());
    let dom = parser.one(html);

    let head = xml_query!(&dom.document, > html > head).ok_or("missing <head>")?;
    // Drop the Content-Type meta tag. Gadget files are typically UTF16 encoded, which this tag
    // specifies. We re-encode it to UTF8.
    head.children
        .borrow_mut()
        .retain(|e| xml_query!(e, meta[http_equiv = "Content-Type"]).is_none());

    // Insert the polyfill.
    head.children.borrow_mut().splice(
        0..0,
        [
            script_node(include_str!("polyfill.js")),
            script_node(format!("window.System.Machine = {};", machine_stats(&sys))),
        ],
    );

    let mut buf = Vec::new();
    html5ever::serialize::serialize(
        &mut buf,
        &SerializableHandle::from(dom.document),
        Default::default(),
    )?;
    Ok(buf)
}

fn script_node(src: impl AsRef<str>) -> Handle {
    let parser = parse_html(RcDom::default(), Default::default());
    let mut dom = parser.one(format!(
        r#"<script type="text/javascript" language="javascript">{}</script>"#,
        src.as_ref()
    ));
    let script =
        xml_query!(&dom.document, > html > head > script).expect("script not found in decoded DOM");
    dom.remove_from_parent(&script);
    script
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
