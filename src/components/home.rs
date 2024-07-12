use leptos::*;

use leptonic::prelude::*;

use uuid::Uuid;

use crate::S3ObjectInfo;
use crate::AppState;
use crate::ImageDisplay;
use crate::SettingsControl;
use crate::ImageListWithFallback;
use crate::Banner;
use crate::DeviceType;
use crate::SiteSettings;
use crate::api::Error;

use crate::local_config::*;
use crate::get_device_type;
use crate::delete_object;
use crate::rename_image;
use crate::update_comment;
use crate::update_app_state;

#[component]
pub fn Home(
  list_image_resource: ReadSignal<Option<Result<Vec<S3ObjectInfo>, Error>>>,
  app_title: ReadSignal<String>,
  app_state_signal: RwSignal<Option<AppState>>,
  set_refetch_list_signal: WriteSignal<bool>,
  public_settings: WriteSignal<SiteSettings>,
  current_image: ReadSignal<Option<String>>,
  set_current_image: WriteSignal<Option<String>>,
  ) -> impl IntoView {
    let (current_config, set_current_config) = create_signal(None::<SiteConfig>);

    let (load_error, set_load_error) = create_signal(None::<String>);

    let update_error = create_rw_signal(None::<String>);

    let toasts = expect_context::<Toasts>();

    create_effect(move |_| {
        match update_error.get() {
            Some(e) => {
                toasts.push(
                    Toast {
                    id: Uuid::new_v4(),
                    created_at: time::OffsetDateTime::now_utc(),
                    variant: ToastVariant::Error,
                    header: "Update Error".into_view(),
                    body: e.into_view(),
                    timeout:  ToastTimeout::DefaultDelay,
                });
                update_error.set(None);
            },
            None => (),
        }
    });

    match get_current_config() {
        Ok(config) => set_current_config.set(Some(config)),
        Err(err) => {
            match err {
                ConfigError::Io => (),
                ConfigError::Serialization => (),
                ConfigError::Encoding => (),
                ConfigError::Storage => {
                    set_load_error.set(Some("Unable to read from browser local storage".to_string()));
                },
                _ => (),

            };
            log::info!("error getting config {:?}", err);
            set_current_config.set(None);
        }
    }

    let (config_updated, _) = create_signal(false);

    let (delete, set_delete) = create_signal(None::<String>);

    let (caption, set_caption) = create_signal(None::<String>);

    create_effect(move |_| {
        config_updated.get(); 
        match get_current_config() {
            Ok(config) => set_current_config.set(Some(config)),
            Err(_) => set_current_config.set(None),
        }
        set_refetch_list_signal.set(true);
    });

    let (current_name, set_current_name) = create_signal( "".to_string());

    let delete_action = create_action(move |_: &String| async move {
        let key = format!("images/{}", delete.get_untracked().expect("delete key expected"));
        if delete_object(key, update_error).await {
            set_refetch_list_signal.set(true)
        }
    });

    create_effect(move |_| match delete.get() {
        Some(_) => delete_action.dispatch("".to_string()),
        None => (),
    });


    let rename_action = create_action(move |_: &String| async move {
        match app_state_signal.get_untracked() {
            Some(app_state) => {
                match list_image_resource.get_untracked() {
                    Some(list_result) => {
                        match list_result {
                            Ok(list) => {
                                rename_image(app_state.current_image_display_name(list), current_name.get_untracked(), update_error).await;
                                set_refetch_list_signal.set(true);
                                set_current_name.set("".to_string());
                            },
                            Err(_) => (),
                        }
                    },
                    None => (),
                }
            },
            None => (),
        }
    });

    let update_caption_action = create_action(move |_: &String| async move {
      let list = list_image_resource.get_untracked().expect("image list expected").expect("image list result expected");
      let e_tag = app_state_signal.get_untracked().expect("app state expected").e_tag(list);
      update_comment(caption.get_untracked().expect("caption expected"), e_tag, update_error).await;
      let mut app_state = app_state_signal.get_untracked().expect("app state expected");
      app_state.current_caption = Some(caption.get_untracked().expect("caption expected"));
      app_state_signal.set(Some(app_state));
      set_refetch_list_signal.set(true);
    });

    // set caption listener
    create_effect(move |_| {
      match caption.get() {
        Some(new_caption) => {
          let existing_caption = app_state_signal.get_untracked().expect("app state expected").current_caption;
          if existing_caption != Some(new_caption) {
            update_caption_action.dispatch("".to_string());
          }
        },
        None => (),
      }
    });

    // rename listener
    create_effect(move |_| {
        let cn = current_name.get();
        match app_state_signal.get_untracked() {
            Some(app_state) => {
                let list = list_image_resource.get().expect("list expected").expect("result expected");
                let current_filename = app_state.current_image_display_name(list);
                if cn != "" && current_filename != cn {
                    rename_action.dispatch("effect".to_string());
                }
            },
            None => (),
        };
    });

    //current image listener
    create_effect(move |_| {
        //log::info!("CURRENT IMAGE LISTENER");
        match current_image.get() {
            Some(i) => {
                let list = list_image_resource.get().expect("list expected").expect("result expected");
                match app_state_signal.get_untracked() {
                  Some(mut app_state) => {
                    if app_state.current_image_name(list.clone()) != i {
                      let new_state = app_state.set_current_image(i, list.clone());
                      update_app_state(app_state_signal, new_state, list);
                    }
                  }
                  None => ()
                }
            }
            None => (),
        }
    });


    let (read_current_image, set_read_current_image) = create_signal("".to_string());

    create_effect(move |_| {
        match app_state_signal.get() {
            Some(app_state) => {
                match list_image_resource.get() {
                    Some(result) => {
                        match result {
                            Ok(list) => {
                                set_read_current_image.set(app_state.current_image_name(list));
                            },
                            Err(_) => (),
                        }
                    },
                    None => (),
                }
            },
            None => (),
        }
    });

    view! {
      <div class="weblumapp">
        {view!{ 
            <div>
                {move || match current_config.get() {
                    Some(_) => {
                        view!{
                            <div>
                                <Banner
                                    list_image_resource={list_image_resource}
                                    app_state_signal={app_state_signal}
                                    app_title={app_title}
                                    set_delete={set_delete}
                                    rename={set_current_name}
                                    caption={set_caption}
                                    refresh_image_list={set_refetch_list_signal}
                                    public_settings={public_settings}
                                    update_error={update_error}
                                />
                                <div 
                                    style = {match get_device_type() {
                                        DeviceType::Desktop => "display:flex; flex-direction: row;",
                                        DeviceType::Mobile => "display:flex; flex-direction: column;",
                                    }}
                                >
                                <ImageListWithFallback
                                    list_image_resource={list_image_resource}
                                    current_image={set_current_image}
                                    read_current_image={read_current_image}
                                />
  
                                <ImageDisplay 
                                    app_state_signal={app_state_signal}
                                    list_image_resource={list_image_resource}
                                />
                            </div>
                        </div>
                    }
                },
                None => view!{
                    <div>
                        <div>
                            <h4>"No site configuration found."</h4>
                            <div>{move || load_error.get()}</div>
                            <div>"Use the button below to configure a storage location for your photos."</div>
                            <SettingsControl />
                        </div>
                    </div>},
                }}
            </div>
            }
        }
        </div>
    }
}
