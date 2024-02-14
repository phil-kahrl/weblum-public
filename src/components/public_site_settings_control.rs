use leptos::*;
use leptonic::prelude::*;
use leptos_icons::RiIcon::RiFileSettingsDocumentLine;
use crate::SiteSettings;
use crate::set_public_site_settings;
use crate::get_public_site_settings;

#[component]
pub fn EditPublicSiteSettingsControl(
    settings_updated: WriteSignal<SiteSettings>,
    update_error: RwSignal<Option<String>>,
) -> impl IntoView {

    let (show_modal, set_show_modal) = create_signal(false);

    let (page_title_input, set_page_title_input) = create_signal("".to_string());
    let (app_title_input, set_app_title_input) = create_signal("".to_string());

    let (original_page_title, set_original_page_title) = create_signal("".to_string());
    let (original_app_title, set_original_app_title) = create_signal("".to_string());

    let fetch_public_site_settings = create_action(move |_: &String| async move {
        let s = get_public_site_settings().await;
        set_page_title_input.set(s.page_title.clone());
        set_app_title_input.set(s.app_title.clone());
        set_original_page_title.set(s.page_title.clone());
        set_original_app_title.set(s.app_title.clone());
    });

    let update_public_site_settings = create_action(move |_| async move {
        update_error.set(None);
        let mut settings: SiteSettings = SiteSettings::new();
        settings.page_title = page_title_input.get_untracked();
        settings.app_title = app_title_input.get_untracked();
        match set_public_site_settings(settings.clone()).await {
            Ok(_settings) => {
                settings_updated.set(settings);
            },
            Err(_) => {
                update_error.set(Some("error updating public settings".to_string()));
            },
        };
    });

    create_effect(move |_| {
        if show_modal.get() {
            fetch_public_site_settings.dispatch("".to_string());
        }
    });

    view!{
        <div>
            <div title = "Update Public Site Settings">
                <Button
                    variant=ButtonVariant::Flat
                    disabled = {false}
                    on_click=move |_ev| {
                        set_show_modal.set(true);
                    }
                >
                    <div>
                        <Icon icon=leptos_icons::Icon::from(RiFileSettingsDocumentLine) />
                        <div class="iconButtonText">"Public Settings"</div>
                    </div>
                </Button>
            </div>

            <Modal show_when = {show_modal}>
                <ModalHeader><ModalTitle>"Update Public Site Settings"</ModalTitle></ModalHeader>
                <ModalBody>
                    <div>
                        <div>"Page Title"</div>
                        <TextInput
                            get = {page_title_input}
                            set = {set_page_title_input}
                        />
                    <div>"App Title"</div>
                        <TextInput
                            get = {app_title_input}
                            set = {set_app_title_input}
                        />
                    </div>
                </ModalBody>
                <ModalFooter>
                    <ButtonWrapper>
                        <Button
                            disabled = Signal::derive( move || {
                                original_page_title.get() == page_title_input.get() &&
                                original_app_title.get() == app_title_input.get()
                            })
                            on_click=move |_| {
                                log::info!("Save settings");
                                update_public_site_settings.dispatch("");
                                set_show_modal.set(false);
                            }
                            color=ButtonColor::Primary
                        >
                            "Save Settings"
                        </Button>
                        <Button
                            on_click=move |_| {
                                log::info!("cancel");
                                set_show_modal.set(false);
                            }
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
