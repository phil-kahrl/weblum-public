use leptos::*;

use weblumclient::*;
use leptonic::prelude::*;

use wasm_bindgen::prelude::wasm_bindgen;

pub fn main() {

    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|| view! {
        <Root default_theme=LeptonicTheme::default()>
            <App />
        </Root>
    }.into_view())
}
