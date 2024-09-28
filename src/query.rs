use markup5ever_rcdom::{Handle, NodeData};

pub fn get_name<'a>(n: &'a Handle) -> Option<&'a str> {
    match &n.data {
        NodeData::Element { name, .. } => Some(name.local.as_ref()),
        _ => None,
    }
}

pub fn get_attr<'a>(n: &'a Handle, key: &str) -> Option<String> {
    let attrs = match &n.data {
        NodeData::Element { attrs, .. } => attrs.borrow(),
        _ => return None,
    };
    attrs
        .iter()
        .filter(|a| a.name.local.as_ref() == key)
        .next()
        .map(|val| val.value.as_ref().to_string())
}

pub fn get_text_contents(n: &Handle) -> Option<String> {
    let strings = n
        .children
        .borrow()
        .iter()
        .filter_map(|c| match &c.data {
            NodeData::Text { contents } => Some(contents.borrow().as_ref().to_string()),
            _ => None,
        })
        .collect::<Vec<String>>();
    if strings.is_empty() {
        None
    } else {
        Some(strings.join(""))
    }
}

macro_rules! xml_query {
    ($elem:expr, ) => {
        Some($elem)
    };

    ($elem:expr, $name:ident $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::get_name(e) == Some(stringify!($name)))
            .and_then(|e| xml_query!(e, $($t)*))
    };

    ($elem:expr, > $($t:tt)+) => {
        $elem.children.borrow().iter().cloned()
            .filter_map(|e| xml_query!(e, $($t)+))
            .next()
    };

    ($elem:expr, [$k:ident=$v:expr] $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::get_attr(e, &stringify!($k).replace('_', "-"))
                .map(|v| v == $v)
                .unwrap_or(false))
            .and_then(|e| xml_query!(e, $($t)*))
    };

    ($elem:expr, [$k:ident~=$v:expr] $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::get_attr(e, &stringify!($k).replace('_', "-"))
                .map(|v| v.to_lowercase() == $v.to_lowercase())
                .unwrap_or(false))
            .and_then(|e| xml_query!(e, $($t)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HTML: &'static str = r#"
        <html>
            <head>
                <meta http-equiv="Content-Type" value="Unicode" />
                <title>Foo</title>
            </head>
        </html>
    "#;

    #[test]
    fn test_macro_compiles_html5ever() {
        use html5ever::{driver::parse_document, tendril::TendrilSink};
        use markup5ever_rcdom::RcDom;

        let parser = parse_document(RcDom::default(), Default::default());
        let dom = parser.one(TEST_HTML).document;
        assert!(matches!(dom.data, NodeData::Document));

        xml_query!(&dom, > html > head).unwrap();
        xml_query!(&dom, > html > head > title).unwrap();
        xml_query!(&dom, > html > head > meta[http_equiv="Content-Type"]).unwrap();
        xml_query!(&dom, > html > head > meta[http_equiv~="content-type"]).unwrap();
    }

    #[test]
    fn test_macro_compiles_xml5ever() {
        use markup5ever_rcdom::RcDom;
        use xml5ever::{driver::parse_document, tendril::TendrilSink};

        let parser = parse_document(RcDom::default(), Default::default());
        let dom = parser.one(TEST_HTML).document;

        xml_query!(&dom, > html > head).unwrap();
        xml_query!(&dom, > html > head).unwrap();
        xml_query!(&dom, > html > head > title).unwrap();
        xml_query!(&dom, > html > head > meta[http_equiv="Content-Type"]).unwrap();
        xml_query!(&dom, > html > head > meta[http_equiv~="content-type"]).unwrap();
    }
}
