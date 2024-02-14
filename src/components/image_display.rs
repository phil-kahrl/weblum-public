use leptos::*;

use crate::AppState;
use crate::DeviceType;
use crate::S3ObjectInfo;
use crate::LoadingIndicator;

use leptos_icons::BiIcon::BiDownloadSolid;
use crate::api::Error;

use leptonic::prelude::*;

use crate::update_app_state;
use crate::local_config::get_current_config;

use crate::get_device_type;

#[component]
fn ImageBody (
    image_key: String,
    image_caption: Option<String>,
) -> impl IntoView {
    view!{
        <div>
            <img 
                style={
                    match get_device_type() {
                        DeviceType::Desktop => "max-width: 900px",
                        DeviceType::Mobile => "max-width: 400px",                      
                    }}
                    src={format!("https://s3.{}.amazonaws.com/{}/{}", 
                        get_current_config().expect("region").region,
                        get_current_config().expect("bucket").s3_bucket_name(),
                        image_key
                    )}
            />
            <div>
                {
                    match image_caption {
                        Some(caption) => caption.into_view(),
                        None => "".into_view()
                    }
                }
            </div>
        </div>
    }
}

#[component]
fn ImageHeader(
    image_list: Vec<S3ObjectInfo>,
    app_state_signal: RwSignal<Option<AppState>>,
)
-> impl IntoView {

    let (image_list_signal, _) = create_signal(image_list.clone());

    match app_state_signal.get_untracked() {
        Some(_app_state) => {
            let (app_state, _) = create_signal(_app_state.clone());

            return view!{

                <div 
                    style={
                        match get_device_type() {
                            DeviceType::Desktop => "max-width: 900px; width: 900px;",
                            DeviceType::Mobile => "max-width: 400px",                      
                        }
                    }
                    class="imageButtonsContainer"
                >
                    <Button
                        disabled = Signal::derive(move || {!app_state.get().has_previous_image()})
                        on_click=move |_evt| {
                            let new_app_state = app_state.get().previous_image();
                            update_app_state(app_state_signal, new_app_state, image_list_signal.get());                 
                        }
                    >
                        "Previous"       
                    </Button>

                    <div>
                        <div>
                            {app_state.get_untracked().current_image_display_name(image_list.clone()).into_view()}
                        </div>

                        <a class="link"
                            target={"_blank"} 
                            href={format!("https://s3.{}.amazonaws.com/{}/images/{}",
                                get_current_config().expect("region").region,
                                get_current_config().expect("region").s3_bucket_name(),
                                _app_state.current_image_display_name(image_list.clone())
                            )}>
                            <span title="Download image">
                                <Icon icon=leptos_icons::Icon::from(BiDownloadSolid) />
                            </span>
                        </a>
                    </div>

                    <Button
                        disabled = Signal::derive(move || {!app_state.get().has_next_image(image_list.clone())})
                        on_click={move |_evt| {
                            let new_app_state = app_state.get().next_image(image_list_signal.get());
                            update_app_state(app_state_signal, new_app_state, image_list_signal.get());           
                        }}
                    >
                        "Next"     
                    </Button>
                </div>
            }.into_view()
        },
        None => "no app state".into_view(),
    }
}

#[component]
pub fn ImageDisplay(
    app_state_signal: RwSignal<Option<AppState>>,
    list_image_resource: ReadSignal<Option<Result<Vec<S3ObjectInfo>, Error>>>,
) -> impl IntoView {
    {
        move || match list_image_resource.get() {
            Some(resource) => {
                match resource {
                    Ok(image_list) => {
                        match app_state_signal.get() {
                            Some(app_state) => {
                                return view!{<div>
                                    <ImageHeader
                                        app_state_signal = {app_state_signal} 
                                        image_list={image_list.clone()}
                                    />
                                    <ImageBody 
                                        image_key=app_state.clone().current_image_name(image_list.clone())
                                        image_caption=app_state.clone().current_caption()
                                    />
                                </div>}.into_view();
                            },
                            None => "no app state".into_view(),
                        }
                    },
                    Err(_) => "No image to display.".into_view(),
                }
            },
            None => view!{ <div class="imageDisplay"><LoadingIndicator /></div>}.into_view(),
        }
    }
}