use leptos::*;

use crate::S3ObjectInfo;
use crate::ImageInfo;


#[component]
pub fn ImageSelect(
    list: Vec<S3ObjectInfo>,
    current_image: WriteSignal<Option<String>>,
    read_current_image: ReadSignal<String>,
) -> impl IntoView {
    view!{
        <div style="display:flex; justify-content: center; align-items: center; margin-top: 10px; width: 355px;">
            <select
                style="border: 1px solid black; font-size: 18px; margin: 10px; padding: 10px; width: 350px;"
                on:change={move |evt| {
                    let image_key = event_target_value(&evt);
                    current_image.set(Some(image_key));
                }}
            >
                <For
                    each=move || list.clone()
                    key=|contents| String::from(&contents.key())
                    children=move |contents: S3ObjectInfo| {
                        let mut mut_key = String::from(&contents.key());
                        let name_only = mut_key.split_off(7);
                        view! {
                            <option 
                                value={&contents.key()}
                                selected = move ||  {
                                    // todo use signal array.
                                    read_current_image.get() == String::from(&contents.key())
                                }
                            >
                                {name_only}
                            </option>
                        }.into_view()
                    }
                />
            </select>
        </div>
    }
}