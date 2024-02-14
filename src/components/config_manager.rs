use leptos::*;
use leptonic::prelude::*;
use crate::DeviceType;
use crate::PurchaseControl;
use crate::SiteSelector;
use uuid::Uuid;

use crate::local_config::*;
use crate::get_device_type;

#[component]
pub fn ConfigList(
    sites: ReadSignal<Vec<SiteConfig>>,
    current_site_id: ReadSignal<String>,
    delete_site: WriteSignal<String>,
) -> impl IntoView {

    let (copied_config, set_copied_config) = create_signal( None::<SiteConfig>);

    create_effect( move |_| {
        match copied_config.get() {
            Some(config) => {
                let _ = leptos::window().navigator().clipboard().expect("clipboard exists").write_text(&config.encoded());
            },
        None => ()
        }
    });

    view!{
        <div>
            <TableContainer>
                <Table bordered=false hoverable=true>
                    <Thead>
                        <Tr>
                            <Th min_width=true>""</Th>
                            <Th min_width=true>""</Th>
                            <Th min_width=true>"Bucket"</Th>
                            <Th min_width=true>"Access Key"</Th>
                        </Tr>
                    </Thead>
                        {move || sites.get().to_vec().into_iter()
                            .map(|s| {
                                let (site, _) = create_signal(s);
                                view! {
                                    <Tr>
                                <Td>
                            <Button
                                disabled = Signal::derive(move || {current_site_id.get() == site.get().id})
                                on_click = move |_evt| {
                                    delete_site.set(site.get().id);
                                }
                            >
                                "Delete"
                            </Button>
                        </Td>
                        <Td>
                            <Button
                                disabled = {false}
                                on_click = move |_evt| {
                                    set_copied_config.set(Some(site.get().clone()));
                                }
                            >
                                "Copy"
                            </Button>
                        </Td>
                        <Td>{move || site.get().s3_bucket_name()}</Td>
                        <Td>{move || site.get().access_key}</Td>
                    </Tr>
                }}).collect_view()}
            </Table>
        </TableContainer>
    </div>
  }
}

#[component]
pub fn AddConfig(config_added: WriteSignal<bool>) -> impl IntoView {
    let toasts = expect_context::<Toasts>();
    let (input, set_input) = create_signal( "".to_string());
    let (add, set_add) = create_signal(false);

    create_effect( move |_| {
        if add.get() {
            if input.get().len() > 10 {
                match SiteConfig::from_encoded(input.get_untracked()) {
                    Ok(new_site_config) => {
                        match add_site_config(new_site_config) {
                            Ok(config) => {
                                set_input.set("".to_string());
                                config_added.set(true);
                                let _s_ = set_current_site(config.id);
                                let _ = leptos::window().location().reload();
                            },
                            Err(err) => {
                                toasts.push(
                                    Toast {
                                        id: Uuid::new_v4(),
                                        created_at: time::OffsetDateTime::now_utc(),
                                        variant: ToastVariant::Error,
                                        header: "Add site failed.".into_view(),
                                        body: format!("Unable to add site. {}", err).into_view(),
                                        timeout:  ToastTimeout::DefaultDelay,
                                    }
                                );
                                set_add.set(false)
                            },
                        }
                    },
                    Err(err) => {
                        toasts.push(
                            Toast {
                                id: Uuid::new_v4(),
                                created_at: time::OffsetDateTime::now_utc(),
                                variant: ToastVariant::Error,
                                header: "Add site failed.".into_view(),
                                body: format!("Unable to add site. {}", err).into_view(),
                                timeout:  ToastTimeout::DefaultDelay,
                            }
                        );
                        set_add.set(false)
                    },
                }
            }
            set_add.set(false);
        }
    });

    view!{
        <div>
            <TextInput
                placeholder = "add your site config token here ..."
                get={input}
                set={set_input}
            />
            <Button
                disabled = Signal::derive( move || {input.get().len() < 10})
                on_click=move |_evt| {
                    set_add.set(!add.get_untracked());
                }
            >
                "Add"
            </Button>
        </div>
    }
}

#[component]
pub fn EditConfig() 
-> impl IntoView {
    let default_sites = match get_sites() {
        Ok(sites) => sites,
        Err(_) => vec![],
    };

    let (sites, set_sites) = create_signal(default_sites);

    let (current_site_id, set_current_site_id) = create_signal("".to_string());

    match get_current_config() {
        Ok(config) => {
            set_current_site_id.set(config.id);
        },
        Err(_) => (),
    };

    let (delete_site, set_delete_site) = create_signal("".to_string());

    let (config_added, set_config_added) = create_signal(false);

    create_effect( move |_| {
        if config_added.get() {
            match get_sites() {
                Ok(sites) => set_sites.set(sites),
                Err(_) => set_sites.set(vec![]),
            };
            set_config_added.set(false);
        }
    });

    create_effect(move |_| {
        if delete_site.get().len() > 1 {
            let _ = remove_site(delete_site.get());
            match get_sites() {
                Ok(_sites) => {
                    set_sites.set(_sites);
                },
                Err(_) => set_sites.set(vec![]),
            };
            set_delete_site.set("".to_string());
        }
    });

    view! {
        <div>
            <ConfigList
                sites={sites}
                current_site_id={current_site_id}
                delete_site={set_delete_site}
            />
            <AddConfig config_added={set_config_added} />    
        </div>
    }
}


#[component]
pub fn ConfigManager() 
  -> impl IntoView {

    let (site_updated, set_site_updated) = create_signal( false);

    create_effect(move |_| {
        if site_updated.get() {
            let _ = leptos::window().location().reload();
        }
    });

    view!{
        <div
            style = {match get_device_type() {
                DeviceType::Mobile => "width: 350px; min-width: 350px; min-height: 450px;",
                DeviceType::Desktop => "width: 650px; min-width: 650px; min-height: 450px;",
            }}
        >
            <Tabs>
                <Tab name="tab-1" label="Purchase Site".into_view() >
                    <PurchaseControl />
                </Tab>
                <Tab name="tab-2" label="Your Sites".into_view() >
                    <SiteSelector set_site_updated = {set_site_updated} />
                    <EditConfig />
                </Tab>
            </Tabs>
        </div>
    }
}