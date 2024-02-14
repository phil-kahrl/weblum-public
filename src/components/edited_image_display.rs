use leptos::*;
use crate::encode_binary;

// Component to display the actual image being edited.
#[component]
pub fn EditedImageDisplay(
    file_binary: ReadSignal<Vec<u8>>,
) -> impl IntoView {
    view!{
        <div>
            {move || {
                view!{<img src={format!("data:image/jpeg;base64,{}", encode_binary(file_binary.get()))} />}
            }}
        </div>
    }
}