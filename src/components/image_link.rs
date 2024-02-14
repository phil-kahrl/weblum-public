use leptos::*;
use crate::S3ObjectInfo;
use crate::ImageInfo;

#[component]
pub fn ImageLink(
    contents: S3ObjectInfo,
    set_current_image: WriteSignal<Option<String>>,
    current: RwSignal<bool>,
) -> impl IntoView
{
    let mut mut_key_to = String::from(&contents.key());
    let name_only = mut_key_to.split_off(7);
    let mut date_display = String::from(&contents.last_modified());
    date_display.truncate(10);
    let class_name = "link";

    view! {
        <div 
            class={class_name}
            on:click={move |_evt| {
                set_current_image.set(Some(contents.clone().key()));
            }}
            >
                <div class=
                    { move || 
                        if current.get() { 
                            "link selected"
                        }
                        else { 
                            "link"
                        }
                    }
                >
                    <div>{format!("{}", String::from(&name_only))}</div>
                    <div class="imageDate">{format!("{}", String::from(&date_display))}</div>
                </div>
            </div>
    }
}