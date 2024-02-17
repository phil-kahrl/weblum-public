use leptos::*;
use leptonic::prelude::*;
use crate::local_config::get_current_config;
use crate::ConfigManager;

#[component]
pub fn UploadFileControl(
    start_upload: WriteSignal<Option<String>>,
    default_upload_filename: ReadSignal<String>,
    disabled: ReadSignal<bool>,

) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let (filename, set_filename) = create_signal(default_upload_filename.get_untracked());

    let current_config = get_current_config().expect("config expected");

    let current_config_can_upload = current_config.access_key.is_some()
        && current_config.secret_key.is_some();

    create_effect(move |_| {
        //log::info!("upload effect");
        if default_upload_filename.get() != filename.get_untracked() {
            set_filename.set(default_upload_filename.get_untracked());
        }
    });

    view!{
        <div>
            <div title = "Upload Image" style="margin-right: 50px;">
                <Button
                    disabled = {disabled}
                    on_click=move |_ev| {
                        set_show_modal.set(true);
                    }
                >
                    <div>
                        <div>{"Upload"}</div>
                    </div>
                </Button>
            </div>

            <Modal show_when = {show_modal}>
                <ModalHeader><ModalTitle>"Upload Image"</ModalTitle></ModalHeader>
                <ModalBody>
                    <div>
                        {if current_config_can_upload {"".into_view()} else {
                            view!{
                                <div> 
                                    <div>"Credentials Required for to upload content. Set or purchase credentials below."</div>        
                                    <ConfigManager />
                                </div>
                            }.into_view()
                        }}
                    </div>
                    <div>
                    {if current_config_can_upload {
                        view!{ 
                            <div>
                                <div>"Name"</div>
                                <TextInput
                                    get = {filename}
                                    set = {set_filename}
                                />
                            </div>
                        }.into_view()
                    } else {
                        "".into_view()
                    }}
                    </div>
                </ModalBody>
                <ModalFooter>
                    <ButtonWrapper>
                        <Button
                            disabled = {!current_config_can_upload}
                            on_click = move |_| {
                                set_filename.set(filename.get_untracked());
                                start_upload.set(Some(filename.get_untracked()));
                                set_show_modal.set(false);
                            }
                            color=ButtonColor::Primary
                        >
                            "Upload"
                        </Button>
                        <Button 
                            on_click=move |_| {
                                set_show_modal.set(false);                          }
                            color=ButtonColor::Secondary
                        >
                            "Cancel"
                        </Button>
                    </ButtonWrapper>
                </ModalFooter>
            </Modal>
        </div>
    }
}