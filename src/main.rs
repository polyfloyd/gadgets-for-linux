use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use webkit6::{prelude::*, WebView};

fn activate(application: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(application);
    window.init_layer_shell();

    window.set_layer(Layer::Bottom);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_margin(Edge::Right, 20);
    window.set_margin(Edge::Bottom, 20);

    let webview = WebView::new();
    webview.load_uri("https://crates.io/");
    window.set_child(Some(&webview));

    window.show();

    webview.evaluate_javascript(
        "alert('Hello');",
        None,
        None,
        gtk4::gio::Cancellable::NONE,
        |_result| {},
    );
}

fn main() {
    let application = gtk4::Application::new(Some("sh.wmww.gtk-layer-example"), Default::default());

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}
