mod api;
mod state;
mod screens;

use dioxus::prelude::*;
use screens::App;

fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Blog Manager")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1280.0, 800.0))
                        .with_min_inner_size(dioxus::desktop::LogicalSize::new(900.0, 600.0)),
                )
        )
        .launch(App);
}
