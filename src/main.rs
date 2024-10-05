use clap::{arg, command, value_parser};
use gadget::Gadget;
use gtk4::gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::borrow::Cow;
use std::cell::RefCell;
use std::error::Error;
use std::path::{self, Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use tempfile::tempdir;
use webkit6::{prelude::*, Settings, WebContext, WebView};

#[macro_use]
mod query;
mod gadget;
mod webhack;

fn app_main(
    application: &gtk4::Application,
    working_dir: impl AsRef<Path>,
    gadget_files: &[impl AsRef<Path>],
) -> Result<(), Box<dyn Error>> {
    let working_dir = working_dir.as_ref();

    let cssp = gtk4::CssProvider::new();
    cssp.load_from_string(r#"window.background { background: unset; }"#);
    let display = gtk4::gdk::Display::default().unwrap();
    gtk4::style_context_add_provider_for_display(
        &display,
        &cssp,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let web_settings = Settings::new();
    web_settings.set_enable_write_console_messages_to_stdout(true);

    let gadget_widgets: Vec<WebView> = gadget_files
        .iter()
        .map(|f| -> Result<WebView, Box<dyn Error>> {
            let f = f.as_ref();
            let mut gadget = Gadget::from_file(f)?;
            eprintln!("loaded gadget: {}", &gadget);

            let f_name = f
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or(Cow::Borrowed("unnamed"));
            let web_root = working_dir.join(&*f_name).to_string_lossy().to_string();

            gadget.unpack_to(&*web_root)?;

            let web_context = WebContext::new();
            web_context.add_path_to_sandbox(working_dir, true);
            let web_view = WebView::builder()
                .settings(&web_settings)
                .web_context(&web_context)
                .sensitive(false)
                .build();
            web_view.set_background_color(&gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
            web_view.load_uri(&format!("file://{}/index.html", web_root));

            Ok(web_view)
        })
        .collect::<Result<_, _>>()?;

    for gadget_widget in &gadget_widgets {
        let window = gtk4::ApplicationWindow::builder()
            .application(application)
            .decorated(false)
            .resizable(false)
            .can_focus(false)
            .default_width(1)
            .default_height(1)
            .child(gadget_widget)
            .build();
        let window = Rc::new(window);

        window.init_layer_shell();
        window.set_layer(Layer::Bottom);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Top, true);

        make_window_movable(&window);

        window.present();
    }

    let ctx = gtk4::glib::MainContext::default();
    ctx.spawn_local(async move {
        let mut sys = sysinfo::System::new_all();
        loop {
            async_std::task::sleep(Duration::from_millis(1000)).await;

            sys.refresh_memory();
            sys.refresh_cpu_usage();

            for web_view in &gadget_widgets {
                let (w, h) = web_body_size(&web_view).await;
                web_view.set_size_request(w, h);

                webhack::update_machine_stats(&web_view, &sys).await;
            }
        }
    });

    Ok(())
}

fn main() -> gtk4::glib::ExitCode {
    let cli = command!()
        .arg(
            arg!(-g --gadget <FILE> "path to one ore more .gadget files")
                .required(true)
                .num_args(1..)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(--debug "Debug mode"));

    let app_id = concat!("com.github.polyfloyd.", env!("CARGO_PKG_NAME"));
    let application = gtk4::Application::new(
        Some(app_id),
        ApplicationFlags::default() | ApplicationFlags::HANDLES_COMMAND_LINE,
    );

    let tmp = tempdir().unwrap();

    let working_dir = tmp.path().to_path_buf();
    application.connect_command_line(move |app, args| {
        let matches = cli.clone().get_matches_from(args.arguments());

        let gadget_files: Vec<&PathBuf> = matches.get_many::<PathBuf>("gadget").unwrap().collect();
        let debug = matches.get_flag("debug");

        let wd = if debug {
            &path::absolute("debug").unwrap()
        } else {
            &working_dir
        };

        if let Err(err) = app_main(app, &wd, &gadget_files) {
            eprintln!("{}", err);
            1
        } else {
            0
        }
    });

    application.run()
}

async fn web_body_size(web_view: &WebView) -> (i32, i32) {
    let js = r#"
        return new Promise((resolve, reject) => {
            resolve({w: document.body.offsetWidth, h: document.body.offsetHeight });
        });
    "#;
    let rs = web_view
        .call_async_javascript_function_future(js, None, None, None)
        .await;

    let v = rs.unwrap();
    assert!(v.is_object());
    let w = v.object_get_property("w").unwrap().to_double();
    let h = v.object_get_property("h").unwrap().to_double();

    (w as i32, h as i32)
}

fn make_window_movable(window_rc: &Rc<gtk4::ApplicationWindow>) {
    let delta_prev_rc = Rc::new(RefCell::new(None));

    let click = gtk4::GestureClick::builder().button(0).build();
    let delta_prev = Rc::clone(&delta_prev_rc);
    click.connect_pressed(move |_ev, _npress, x, y| {
        *delta_prev.borrow_mut() = Some((x, y));
    });
    let delta_prev = Rc::clone(&delta_prev_rc);
    click.connect_released(move |_ev, _npress, _x, _y| {
        *delta_prev.borrow_mut() = None;
    });
    window_rc.add_controller(click);

    let mc = gtk4::EventControllerMotion::default();
    let delta_prev = delta_prev_rc;
    let window = Rc::clone(&window_rc);
    mc.connect_motion(move |_event, x, y| {
        let mut prev = delta_prev.borrow_mut();
        let (dx, dy) = match *prev {
            None => return,
            Some((lx, ly)) => (x - lx, y - ly),
        };
        *prev = Some((x, y));

        let new_x = (window.margin(Edge::Left) as f64 + dx) as i32;
        let new_y = (window.margin(Edge::Top) as f64 + dy) as i32;
        window.set_margin(Edge::Left, new_x.max(0));
        window.set_margin(Edge::Top, new_y.max(0));
    });
    window_rc.add_controller(mc);
}
