use clap::{arg, command, value_parser};
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::error::Error;
use std::path::{self, Path, PathBuf};
use tempfile::tempdir;
use webkit6::{prelude::*, Settings, WebContext, WebView};

mod gadget;
mod webhack;

fn app_main(
    application: &gtk4::Application,
    working_dir: &Path,
    gadget_file: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut gadget = gadget::Gadget::from_file(gadget_file)?;
    eprintln!("loaded gadget: {}", &gadget);

    gadget.unpack_to(working_dir)?;

    let cssp = gtk4::CssProvider::new();
    cssp.load_from_string(r#"window.background { background: unset; }"#);
    let display = gtk4::gdk::Display::default().unwrap();
    gtk4::style_context_add_provider_for_display(
        &display,
        &cssp,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = gtk4::ApplicationWindow::builder()
        .application(application)
        .decorated(false)
        .resizable(false)
        .can_focus(false)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Bottom);
    window.set_anchor(Edge::Right, true);
    window.set_anchor(Edge::Bottom, true);
    window.set_margin(Edge::Right, 20);
    window.set_margin(Edge::Bottom, 20);

    let web_context = WebContext::new();
    web_context.add_path_to_sandbox(working_dir, true);
    let web_settings = Settings::new();
    web_settings.set_enable_write_console_messages_to_stdout(true);

    let web_view = WebView::builder()
        .settings(&web_settings)
        .web_context(&web_context)
        .build();
    web_view.set_background_color(&gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
    window.set_child(Some(&web_view));

    web_view.load_uri(&format!(
        "file://{}/index.html",
        working_dir.to_string_lossy()
    ));

    window.set_visible(true);

    Ok(())
}

fn main() {
    let cli = command!()
        .arg(
            arg!(-g --gadget <FILE> "path to a .gadget file")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(--debug  "Debug mode"));

    let app_id = concat!("com.github.polyfloyd.", env!("CARGO_PKG_NAME"));
    let application = gtk4::Application::new(
        Some(app_id),
        ApplicationFlags::default() | ApplicationFlags::HANDLES_COMMAND_LINE,
    );

    let tmp = tempdir().unwrap();

    let working_dir = tmp.path().to_path_buf();
    application.connect_command_line(move |app, args| {
        let matches = cli.clone().get_matches_from(args.arguments());

        let gadget_file = matches.get_one::<PathBuf>("gadget").unwrap().clone();
        let debug = matches.get_flag("debug");

        let wd = if debug {
            &path::absolute("debug").unwrap()
        } else {
            &working_dir
        };

        if let Err(err) = app_main(app, &wd, &gadget_file) {
            eprintln!("{}", err);
            1
        } else {
            0
        }
    });

    application.run();
}
