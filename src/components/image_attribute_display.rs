use leptos::*;

use crate::ImageMetadata;

#[component]
pub fn SingleDisplay (
    im: ImageMetadata,
) -> impl IntoView {
    view!{
        <div style="width: 500px; max-width: 500px; overflow: hidden; ">
            <div style="display: flex; flex-direction: column">
                <div>
                    {im.clone().dimensions.expect("dims should exist").0} 
                        x 
                    {im.clone().dimensions.expect("dims should exist").1}
                </div>

                <div>
                    {format!("{0:.2} Mb", im.size/(1000000.0))}
                </div>

                <div style="display: flex">
                    <label style="font-weight: bold; padding: 5px">{im.clone().mime_type}</label>
                </div>

                <div style="display: flex">
                    <label style="font-weight: bold; padding: 5px">{format!("{}", im.clone().last_modified)}</label>
                </div>

                <div style="display: flex; flex-direction: column">
                    <div style="padding: 5px">{im.clone().get_lat()}</div>
                </div>

                <div style="display: flex; flex-direction: column">
                    <div style="padding: 5px">{im.clone().get_long()}</div>
                </div>

                <div>
                    <div style="display: flex">
                        <label style="font-weight: bold; padding: 10px 0 10px 0">"EXIF Tags"</label>
                    </div>

                    {
                        im.clone().all_tags.into_iter()
                            .map(|contents| {
                                view!{
                                    <div style="display:flex; flex-direction: row; padding: 0 0 0 5px">
                                        <div style="font-weight: bold; padding: 0 5px 0 0">{format!("{} : ",contents.0)}</div>
                                        <div>{contents.1}</div>
                                    </div>
                                }
                            }).collect_view()
                    }
                </div>
            </div>
        </div>
}}

#[component]
pub fn ImageMetaDataDisplay (
    image_metadata: ReadSignal<Option<ImageMetadata>>,
    edited_image_metadata: ReadSignal<Option<ImageMetadata>>,
) -> impl IntoView { view!{
        <div style="display: flex; flex-direction: row; justify-content: space-between; ">
            <div style="padding: 10px;">
                <h3>"Original"</h3>
                <div>
                    {move || match image_metadata.get() {
                        Some(im) => view!{
                            <div>
                                <SingleDisplay im={im} />
                            </div>
                        },
                        None => view!{<div>"No attributes to display"</div>},
                    }}
                </div>
            </div>
            <div style="padding: 10px;">
                <h3>"Edited"</h3>
                <div>
                    {move || match edited_image_metadata.get() {
                        Some(im) => view!{
                            <div>
                                <SingleDisplay im={im} />
                            </div>
                        },
                        None => view!{<div>"No attributes to display"</div>},
                    }}
                </div> 
            </div>
        </div> 
}}