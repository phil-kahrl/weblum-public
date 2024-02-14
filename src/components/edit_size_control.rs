use leptos::*;
use leptonic::prelude::*;
use leptos_icons::TbIcon::*;
use crate::SizeInputs;

#[component]
pub fn EditSizeControl(
    resize_listener: WriteSignal<SizeInputs>,
    disabled: Signal<bool>,

) -> impl IntoView {
    let (show_modal, set_show_modal) = create_signal(false);
    let (width_input, set_width_input) = create_signal(1400.0);
    let (quality, set_quality) = create_signal(50.0);

    view!{
        <div>
            <div title = "Resize Image">
                <Button
                    disabled = disabled
                    on_click=move |_ev| {
                        set_show_modal.set(true);
                    }
                >
                    <div>
                        <Icon icon=leptos_icons::Icon::from(TbResize) />
                        <div style="font-size: .5em">"Resize"</div>
                    </div>
                </Button>
            </div>

            <Modal show_when = {show_modal}>
                <ModalHeader><ModalTitle>"Resize Image"</ModalTitle></ModalHeader>
                <ModalBody>
                    <div>
                        <div>"Resize Image to width: "</div>
                        <NumberInput
                            get = {width_input}
                            set = {set_width_input}
                        />
                    <div>"Use Image quality"</div>
                        <NumberInput
                            get = {quality}
                            set = {set_quality}
                        />
                    </div>
                </ModalBody>
                <ModalFooter>
                    <ButtonWrapper>
                        <Button 
                            on_click=move |_| {
                                let to_update = SizeInputs::new(
                                    (width_input.get_untracked() as u32, width_input.get_untracked() as u32),
                                     quality.get_untracked() as u8,
                                );
                                resize_listener.set(to_update);
                                set_show_modal.set(false);
                            }
                            color=ButtonColor::Primary
                        >
                            "Resize"
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
        </div>
    }
}
