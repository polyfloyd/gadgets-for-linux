use std::iter::Iterator;

pub trait Node {
    fn name(&self) -> &str;
    fn attr(&self, key: &str) -> Option<&str>;
    fn children(&self) -> impl Iterator<Item = &Self>;
    fn children_mut(&mut self) -> impl Iterator<Item = &mut Self>;
}

impl Node for xmltree::Element {
    fn name(&self) -> &str {
        &self.name
    }

    fn attr(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    fn children(&self) -> impl Iterator<Item = &Self> {
        self.children.iter().filter_map(|e| e.as_element())
    }

    fn children_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        self.children.iter_mut().filter_map(|e| e.as_mut_element())
    }
}

macro_rules! xml_query {
    ($elem:expr, ) => {
        Some($elem)
    };

    ($elem:expr, $name:ident $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::Node::name(*e) == stringify!($name))
            .and_then(|e| xml_query!(e, $($t)*))
    };

    ($elem:expr, > $($t:tt)+) => {
        crate::query::Node::children($elem)
            .filter_map(|e| xml_query!(e, $($t)+)).next()
    };

    ($elem:expr, [$k:ident=$v:expr] $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::Node::attr(*e, &stringify!($k).replace('_', "-"))
                .map(|v| v == $v)
                .unwrap_or(false))
            .and_then(|e| xml_query!(e, $($t)*))
    };

    ($elem:expr, [$k:ident~=$v:expr] $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::Node::attr(*e, &stringify!($k).replace('_', "-"))
                .map(|v| v.to_lowercase() == $v.to_lowercase())
                .unwrap_or(false))
            .and_then(|e| xml_query!(e, $($t)*))
    };
}

macro_rules! xml_query_mut {
    ($elem:expr, ) => {
        Some($elem)
    };

    ($elem:expr, $name:ident $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::Node::name(*e) == stringify!($name))
            .and_then(|e| xml_query_mut!(e, $($t)*))
    };

    ($elem:expr, > $($t:tt)+) => {
        crate::query::Node::children_mut($elem)
            .filter_map(|e| xml_query_mut!(e, $($t)+)).next()
    };

    ($elem:expr, [$k:ident=$v:expr] $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::Node::attr(*e, &stringify!($k).replace('_', "-"))
                .map(|v| v == $v)
                .unwrap_or(false))
            .and_then(|e| xml_query!(e, $($t)*))
    };

    ($elem:expr, [$k:ident~=$v:expr] $($t:tt)*) => {
        Some($elem)
            .filter(|e| crate::query::Node::attr(*e, &stringify!($k).replace('_', "-"))
                .map(|v| v.to_lowercase() == $v.to_lowercase())
                .unwrap_or(false))
            .and_then(|e| xml_query_mut!(e, $($t)*))
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macro_compiles() {
        use xmltree::Element;

        let mut dom = Element::parse(
            r#"
                <html>
                    <head>
                        <meta http-equiv="Content-Type" value="Unicode" />
                        <title>Foo</title>
                    </head>
                </html>
            "#
            .as_bytes(),
        )
        .unwrap();

        xml_query!(&dom, > head).unwrap();
        xml_query!(&dom, html > head).unwrap();
        xml_query!(&dom, html > head > title).unwrap();
        xml_query!(&dom, html > head > meta[http_equiv="Content-Type"]).unwrap();
        xml_query!(&dom, html > head > meta[http_equiv~="content-type"]).unwrap();

        xml_query_mut!(&mut dom, > head).unwrap();
        xml_query_mut!(&mut dom, html > head).unwrap();
        xml_query_mut!(&mut dom, html > head > title).unwrap();
        xml_query_mut!(&mut dom, html > head > meta[http_equiv="Content-Type"]).unwrap();
        xml_query_mut!(&mut dom, html > head > meta[http_equiv~="content-type"]).unwrap();
    }
}
