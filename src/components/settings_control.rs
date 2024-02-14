use leptos::*;
use leptonic::prelude::*;
use crate::ConfigManager;
use leptos_icons::AiIcon::AiSettingOutlined;

#[component]
pub fn SettingsControl() -> impl IntoView {
  let (show_modal, set_show_modal) = create_signal(false);

  view!{ 
    <div>
      <div title="Settings">
      <Button
        variant=ButtonVariant::Flat
        on_click =
          move |_evt| {
            set_show_modal.set(true);
          }
      >
        <div>
          <Icon icon=leptos_icons::Icon::from(AiSettingOutlined) />
          <div class="iconButtonText">"Local Settings"</div>
        </div>
      </Button>
      </div>

      <Modal show_when={show_modal}>
       <ModalHeader>
          "Site Publishing Settings"
       </ModalHeader>

       <ModalBody>
          <ConfigManager/>
          <div style="padding: 10px;">
            <Button on_click=move |_evt| set_show_modal.set(false)>
              "Cancel"
            </Button>
          </div>
       </ModalBody>

      </Modal>

    </div>
  }
}