#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::info;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // env_logger::init();

    info!("starting program");
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        shader_version: Some(eframe::egui_glow::ShaderVersion::Es300), // explicit shader version so that egui won't query opengl for supported version.
        ..Default::default()
    };

    eframe::run_native(
        "OR1 Module File Encrypter",
        native_options,
        Box::new(|cc| Box::new(encrypter::EncrypterApp::new(cc))),
    ).expect("error in run native");
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(encrypter::EncrypterApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
