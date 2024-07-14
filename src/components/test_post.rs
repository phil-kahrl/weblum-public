use gloo_net::http::Request;
use leptos::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{EventTarget, File, HtmlInputElement};

#[component]
pub fn TestPost() -> impl IntoView {
    let (file, set_file) = create_signal(Option::<File>::None);

    let upload_image = create_action(move |_: &String| async move {
        let url = "/js/upload";
        let builder = Request::post(&url).body(file.get_untracked());
        let _ = builder.expect("request expected").send().await;
    });


    create_effect(move |_| {
        match file.get() {
            Some(f) => {
                log::info!("File {}", f.name());
                upload_image.dispatch("".to_string());
            },
            None => (),
        }
    });


    
    view!{
        <div>
            <h3>Test Post </h3>
            <form target="_blank" enctype="multipart/form-data" method="POST" action="/">
                <input 
                    type="file" 
                    id="jpeg"
                    name="jpeg"
                    accept="image/jpeg"
                    //on:change=move |ev| {
                        //let t = ev.target();
                        //let et: EventTarget = t.expect("target");
                        //let r: &JsValue = et.as_ref();
                        //let file_input = r.clone().dyn_into::<HtmlInputElement>();
                        //let files = file_input.expect("file input").files();
                        //let file = files.expect("files").item(0).expect("file");
                        //set_file.set(Some(file.clone()));
                        //set_default_upload_filename.set(file.name());
                        //update_image_data.dispatch("".to_string());         
                    //}
                />
                <button type="submit">Submit</button>
            </form>
        </div>
    }
}