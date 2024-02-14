use leptos::*;
use leptonic::prelude::*;

use leptos_icons::AiIcon::*;
use crate::FileEditAndPublishControl;
use crate::SettingsControl;
use crate::EditPublicSiteSettingsControl;
use crate::SiteSettings;
use crate::ConfigManager;
use crate::AppState;
use crate::S3ObjectInfo;

use crate::get_current_config;

#[component]
pub fn UploadModalControl(
    update_required: WriteSignal<bool>,
) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let (update, set_update) = create_signal(false);

    create_effect(move |_| {
        if update.get() {
            update_required.set(true);
            set_show_modal.set(false);
        }
    });

    view!{
        <div>
            <div title = "Add Image">
                <Button
                    variant=ButtonVariant::Flat
                    on_click=move |_ev| {
                        set_show_modal.set(true);
                    }
                >
                    <div>
                        <Icon icon=leptos_icons::Icon::from(AiEditOutlined) />
                        <div class="iconButtonText">"Add Image"</div>
                    </div>
                </Button>
            </div>

            <Modal show_when = {show_modal}>
                <ModalHeader><ModalTitle>"Add Image"</ModalTitle></ModalHeader>
                <ModalBody>
                    <FileEditAndPublishControl update_list={set_update} set_show_modal={set_show_modal} />
                </ModalBody>
                <ModalFooter>
                    <ButtonWrapper>
                        <Button
                            on_click=move |_| {
                                set_show_modal.set(false);
                            }
                            color=ButtonColor::Secondary
                        >
                            "Close"
                        </Button>
                    </ButtonWrapper>
                </ModalFooter>
            </Modal>
        </div>
    }
}


#[component]
pub fn EditCaptionControl(
    update_caption: WriteSignal<Option<String>>,
    app_state_signal: RwSignal<Option<AppState>>,

) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let caption_default = match app_state_signal.get_untracked().expect("").current_caption{
        Some(c) => c,
        None => "".to_string(),
    };

    let (caption, set_caption) = create_signal(caption_default);

    let (can_edit, set_can_edit) = create_signal(false);
    match get_current_config() {
        Ok(config) => {
            if config.access_key.is_some() && config.secret_key.is_some() {
                set_can_edit.set(true);
            }
        },
        Err(_) => ()
    }

    view!{
        <div>
            <div title = "Edit Caption">
                <Button
                    variant=ButtonVariant::Flat
                    on_click=move |_ev| {
                        set_show_modal.set(true);
                    }
                >
                    <div>
                        <Icon icon=leptos_icons::Icon::from(AiEditOutlined) />
                        <div class="iconButtonText">"Edit Caption"</div>
                    </div>
                </Button>
            </div>


            {move || match can_edit.get() {
                true => view!{
                    <Modal show_when = {show_modal}>
                        <ModalHeader><ModalTitle>"Edit Caption"</ModalTitle></ModalHeader>
                        <ModalBody>
                            <div>
                                <div>"Edit Caption : "</div>
                                <TextInput
                                    get = {caption}
                                    set = {set_caption}
                                />
                            </div>
                        </ModalBody>
                        <ModalFooter>
                            <ButtonWrapper>
                                <Button
                                    disabled=Signal::derive(move || {
                                        match app_state_signal.get_untracked() {
                                            Some(app_state) => {
                                                match app_state.current_caption {
                                                    Some(c) => c == caption.get(),
                                                    None => false,
                                                }
                                            }
                                            None => false,
                                        }
                                    })
                                    on_click=move |_| {
                                        update_caption.set(Some(caption.get_untracked()));
                                        set_show_modal.set(false);
                                    }
                                    color=ButtonColor::Primary
                                >
                                    "Update"
                                </Button>
                                <Button 
                                    on_click=move |_| {
                                        set_show_modal.set(false);
                                    }
                                    color=ButtonColor::Secondary
                                >
                                    "Cancel"
                                </Button>
                            </ButtonWrapper>
                        </ModalFooter>
                    </Modal>
                },
                false => view!{
                     <Modal show_when={show_modal}>
                        <ModalHeader>
                            "Credentials Required for Editing Caption"
                        </ModalHeader>
                        <ModalBody>
                            <div style="padding: 10px;">"Set or purchase credentials below"</div>
                            <ConfigManager />
                            <div style="padding: 10px;">
                                <Button on_click=move |_evt| set_show_modal.set(false)>
                                    "Cancel"
                                </Button>
                            </div>
                        </ModalBody>
                    </Modal>
                },
            }}
        </div>
    }
}

#[component]
pub fn RenameImageControl(
    app_state_signal: RwSignal<Option<AppState>>,
    set_current_image_name: WriteSignal<String>,
    image_list: Vec<S3ObjectInfo>,
) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);

    let (name, set_name) = create_signal(app_state_signal.get_untracked().expect("").current_image_display_name(image_list.clone()));
   
    let (can_rename, set_can_rename) = create_signal(false);
    match get_current_config() {
        Ok(config) => {
            if config.access_key.is_some() && config.secret_key.is_some() {
                set_can_rename.set(true);
            }
        },
        Err(_) => ()
    }

    let (read_image_list, _) = create_signal(image_list);
    view!{
        <div>
            <div title = "Rename Image">
                <Button
                    variant=ButtonVariant::Flat
                    on_click=move |_ev| {
                        set_show_modal.set(true);
                    }
                >
                    <div>
                        <Icon icon=leptos_icons::Icon::from(AiEditOutlined) />
                        <div class="iconButtonText">"Rename Image"</div>
                    </div>
                </Button>
            </div>

            { move || match can_rename.get() {
                true => view!{
                    <Modal show_when = {show_modal}>
                        <ModalHeader><ModalTitle>"Rename Image"</ModalTitle></ModalHeader>
                        <ModalBody>
                            <div style="width: 400px">
                                <div>"Rename Image : "</div>
                                <TextInput
                                    get = {name}
                                    set = {set_name}
                                />
                            </div>
                        </ModalBody>
                        <ModalFooter>
                            <ButtonWrapper>
                                <Button
                                    disabled = Signal::derive(move || {
                                        name.get() == app_state_signal.get().expect("").current_image_name(read_image_list.get_untracked())
                                    })
                            
                                    on_click=move |_| {
                                        set_current_image_name.set(name.get_untracked());
                                        set_show_modal.set(false);
                                    }
                                    color=ButtonColor::Primary
                                >
                                    "Rename"
                                </Button>
                                <Button 
                                    on_click=move |_| {
                                        set_show_modal.set(false);
                                    }
                                    color=ButtonColor::Secondary
                                >
                                    "Cancel"
                                </Button>
                            </ButtonWrapper>
                        </ModalFooter>
                    </Modal>
                },
                false => view!{
                    <Modal show_when={show_modal}>
                        <ModalHeader>
                            "Credentials Required for Rename"
                        </ModalHeader>

                        <ModalBody>
                            <div style="padding: 10px;">"Set or purchase credentials below"</div>
                            <ConfigManager />
                                <div style="padding: 10px;">
                                    <Button on_click=move |_evt| set_show_modal.set(false)>
                                        "Cancel"
                                    </Button>
                                </div>
                        </ModalBody>
                    </Modal>
                },
            }}
        </div>
    }
}

#[component]
pub fn PublicLink (
) -> impl IntoView {

     let bucket_name = get_current_config().expect("config expected").s3_bucket_name();

     view!{
        <div title={bucket_name.clone()}>
            <a href={format!("http://{}", bucket_name)} target="_blank" >
                <Button
                    variant=ButtonVariant::Flat
                    on_click=move |_ev| {}
                >
                    <div title="Public Link">
                        <Icon icon=leptos_icons::Icon::from(AiLinkOutlined) />
                        <div class="iconButtonText">"Public Link"</div>
                    </div>
                </Button>
            </a>
        </div>
    }
}

#[component]
pub fn DeleteImageControl (
    delete: WriteSignal<Option<String>>,
    read_image_name: ReadSignal<String>,
) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);

    let delete_action = create_action(move |_: &String| async move {
        delete.set(Some(read_image_name.get_untracked()));
    });

    let (can_delete, set_can_delete) = create_signal(false);
    match get_current_config() {
        Ok(config) => {
            if config.access_key.is_some() && config.secret_key.is_some() {
                set_can_delete.set(true);
            }
        },
        Err(_) => ()
    }

    view!{
        <div>
            <Button
                variant=ButtonVariant::Flat
                on_click=move |_ev| {
                set_show_modal.set(true);
            }>
            <div title="Delete Image">
                <Icon icon=leptos_icons::Icon::from(AiDeleteOutlined) />
                    <div class="iconButtonText">"Delete Image"</div>
            </div>
            </Button>

            <div>
            {move || match can_delete.get() {
                true => view!{
                    <Modal show_when = {show_modal}>
                        <ModalHeader><ModalTitle>"Delete Image"</ModalTitle></ModalHeader>
                        <ModalBody>
                            <div>{format!("Are you sure you want to delete the image: '{}' ?", read_image_name.get_untracked())}</div>
                        </ModalBody>
                        <ModalFooter>
                            <ButtonWrapper>
                                <Button 
                                    on_click=move |_evt| {
                                        delete_action.dispatch("".to_string());
                                        set_show_modal.set(false);
                                    }
                                    color=ButtonColor::Primary
                                >
                                    "Delete"
                                </Button>
                                <Button 
                                    on_click=move |_| {
                                        set_show_modal.set(false);
                                    }
                                    color=ButtonColor::Secondary
                                >
                                    "Cancel"
                                </Button>
                            </ButtonWrapper>
                        </ModalFooter>
                    </Modal>

                },
                false =>  view!{
                     <Modal show_when={show_modal}>
                        <ModalHeader>
                            "Credentials Required for Delete"
                        </ModalHeader>

                        <ModalBody>
                            <div style="padding: 10px;">"Set or purchase credentials below"</div>
                            <ConfigManager />
                                <div style="padding: 10px;">
                                    <Button on_click=move |_evt| set_show_modal.set(false)>
                                        "Cancel"
                                    </Button>
                                </div>
                        </ModalBody>
                    </Modal>
                },
            }}
            </div>
        </div>
    }
}


#[component]
pub fn UpdateErrorDisplay (
    update_error: ReadSignal<Option<String>>
) -> impl IntoView {
    view!{
        <div>
            {move || match update_error.get() {
                Some(e) => e,
                None => "".to_string()
            }}
        </div>
    }
}

#[component]
pub fn ImageDisplayButtons (
    delete: WriteSignal<Option<String>>,
    rename: WriteSignal<String>,
    caption: WriteSignal<Option<String>>,
    refresh_image_list: WriteSignal<bool>,
    public_settings: WriteSignal<SiteSettings>,
    update_error: RwSignal<Option<String>>,
    app_state_signal: RwSignal<Option<AppState>>,
    image_list: Vec<S3ObjectInfo>,

) -> impl IntoView {
    
    {move || match app_state_signal.get() {
        Some(app_state) =>  {  
            let (read_image_name, _) = create_signal(app_state.current_image_display_name(image_list.clone()));
            view!{
                <div style="display: flex; flex-direction: row; justify-content: space-between; width: 500px;">
                <UploadModalControl
                    update_required={refresh_image_list}
                />
                <DeleteImageControl 
                    delete={delete}
                    read_image_name={read_image_name}
                />
                <RenameImageControl
                    image_list={image_list.clone()}
                    app_state_signal={app_state_signal}
                    set_current_image_name={rename}
                />
                <EditCaptionControl
                    app_state_signal={app_state_signal}
                    update_caption={caption}
                />
                <SettingsControl />
                <EditPublicSiteSettingsControl
                    update_error={update_error}
                    settings_updated={public_settings}
                />
                <PublicLink />
            </div>
        }.into_view()},
        None => "".into_view()
    }}
}