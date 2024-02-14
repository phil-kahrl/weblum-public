use leptos::*;
use crate::ImageSelect;
use crate::S3ObjectInfo;
use crate::DeviceType;
use crate::api::Error;
use crate::List;
use crate::get_device_type;

#[component]
pub fn ImageListWithFallback(
  list_image_resource: ReadSignal<Option<Result<Vec<S3ObjectInfo>, Error>>>,
  current_image: WriteSignal<Option<String>>,
  read_current_image: ReadSignal<String>,
) -> impl IntoView {
    view!{
    <div style="width: 25%;">
    {move || match list_image_resource.get() {
        Some(list_result) => {
            match list_result {
                Ok(list) => {
                    let (list_signal, _) = create_signal(list.clone());
                    match get_device_type() {
                    DeviceType::Desktop => view!{
                        <List
                            list={list_signal}
                            current_image={current_image} 
                            read_current_image={read_current_image}
                        />
                    }.into_view(),
                    DeviceType::Mobile => view!{
                        <ImageSelect 
                            list={list.clone()}
                            current_image={current_image} 
                            read_current_image={read_current_image}
                        />
                    }.into_view(),
                }},
                Err(_) => "Error retrieving list".into_view(),
            }
        },
        None => view!{<div title="Waiting" />}.into_view()
    }}
    </div>
}}
       
