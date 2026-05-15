use dioxus::prelude::*;

mod api;
mod components;
mod models;
mod pages;
mod storage;
mod utils;

use pages::*;

#[derive(Routable, PartialEq, Clone)]
enum AppRoute {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[route("/data")]
    Data {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

fn main() {
    dioxus::LaunchBuilder::new().launch(|| {
        rsx! {
            document::Link { rel: "icon", href: asset!("/public/favicon.ico") }
            Router::<AppRoute> {}
        }
    });
}

#[component]
fn Shell() -> Element {
    rsx! {
        nav { style: "z-index: 2;",
            Link { to: AppRoute::Home {}, active_class: "active", "Home" }
            Link { to: AppRoute::Data {}, active_class: "active", "Data" }
        }
        main { Outlet::<AppRoute> {} }
    }
}
