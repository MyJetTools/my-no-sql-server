use dioxus::prelude::*;

mod api;
mod components;
mod models;
mod pages;
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
            Link { to: AppRoute::Home {}, "Home" }
            Link { to: AppRoute::Data {}, "Data" }
        }
        main { Outlet::<AppRoute> {} }
    }
}
