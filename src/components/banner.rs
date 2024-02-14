use leptos::*;
use crate::ImageDisplayButtons;
use crate::AppState;
use crate::SiteSettings;
use crate::SiteSelector;
use crate::S3ObjectInfo;
use crate::api::Error;

#[component]
pub fn Banner(
    app_state_signal: RwSignal<Option<AppState>>,
    app_title: ReadSignal<String>,
    set_delete: WriteSignal<Option<String>>,
    rename: WriteSignal<String>,
    caption: WriteSignal<Option<String>>,
    refresh_image_list: WriteSignal<bool>,
    public_settings: WriteSignal<SiteSettings>,
    update_error: RwSignal<Option<String>>,
    list_image_resource: ReadSignal<Option<Result<Vec<S3ObjectInfo>, Error>>>,
) 
  -> impl IntoView {

    let (site_updated, set_site_updated) = create_signal(false);

    create_effect(move |_| {
        if site_updated.get() {
            let _ = leptos::window().location().reload();
        }
    });

    view! {
        <div>
            <div style="display: flex; justify-content: center;">
                <SiteSelector set_site_updated={set_site_updated} />
            </div>

            <h3 class="banner">{move || app_title.get()}</h3>
            <div style="display:flex; flex-direction: row; justify-content: center">
                {move || match list_image_resource.get() {
                    Some(image_list) => {
                        match image_list {
                            Ok(list) => {
                                view!{
                                    <ImageDisplayButtons 
                                        delete={set_delete}
                                        rename={rename}
                                        caption={caption}
                                        refresh_image_list={refresh_image_list}
                                        public_settings={public_settings}
                                        update_error={update_error}
                                        app_state_signal={app_state_signal}
                                        image_list={list}
                                    />}.into_view()
                            },
                            Err(_) => {
                                view!{
                                    <div>
                                        "Error loading image list"
                                    </div>
                                }.into_view()
                            }
                        }
                    },
                    None => view!{<div title="Waiting" />}.into_view()
                }
            }            
          </div>
        </div>
      }.into_view()
}