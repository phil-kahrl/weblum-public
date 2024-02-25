use leptos::*;

use crate::local_config::SiteConfig;
use crate::local_config::get_sites;
use crate::get_current_config;
use crate::local_config::set_current_site;

#[component]
pub fn SiteSelector(
    set_site_updated: WriteSignal<bool>,
) -> impl IntoView {

    let current_config_option: Option<SiteConfig> = None;
    let (current_config, set_current_config) = create_signal(current_config_option);

    let site_config_default: Vec<SiteConfig> = vec![];
    let (sites, set_sites) = create_signal(site_config_default);

    let input_default: Option<String> = None;
    let (input, set_input) = create_signal(input_default);

    match get_sites() {
        Ok(value) => set_sites.set(value),
        Err(_) => (),
    }

    match get_current_config() {
        Ok(value) => set_current_config.set(Some(value)),
        Err(_) => (),
    }

    create_effect(move |_| {
        match input.get() {
            Some(s) => {
                match set_current_site(s) {
                    Ok(_) => set_site_updated.set(true),
                    Err(_) => log::info!("SET CURRENT SITE FAILED"),
                }
            },
            None => (),
        }
    });

    { move || match sites.get().len() > 0 {
        true => 
            view!{
                <div style="display: flex; flex-direction: row;">
                    <div style="font-weight: bold; padding: 0 10px 0 10px;">"Current Site"</div>
                    <select style="font-size: .8em;"
                        on:change={move |evt| {
                            let value = event_target_value(&evt);
                            set_input.set(Some(value));
                        }}
                    >
                        {move || sites.get().into_iter()
                            .map(|contents|
                                view! {
                                    <option 
                                        value={contents.clone().id}
                                        selected = {current_config.get().expect("config expected").id == contents.clone().id}
                                    >
                                        {format!("{}", contents.clone().s3_bucket_name())}
                                    </option>
                                }
                            ).collect::<Vec<_>>()
                        }
                    </select>
                </div>
            }.into_view(),
        false => "".into_view(),      
    }}
}
