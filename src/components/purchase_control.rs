use leptos::*;
use leptonic::prelude::*;

use crate::AWSCredentials;
use crate::local_config::SiteConfig;
use crate::decode_string;
use serde::Deserialize;
use serde::Serialize;
use crate::DeviceType;

use leptos_icons::CgIcon::CgSpinner;
use uuid::Uuid;

use gloo_net::http::Request;
use gloo_net::http::RequestBuilder;
use gloo_net::http::Method;
use gloo_net::http::Headers;

use crate::local_config::set_current_site;
use crate::local_config::add_site_config;
use crate::get_device_type;

#[derive(Clone, Deserialize, Serialize)]
pub struct Product {
    id: String,
}

impl Product {
    fn new(id: String) -> Self {
        Self {
            id,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Purchase {
    product: Product,
    token: String,
    #[serde(rename(deserialize = "postalCode"))]
    #[serde(rename(serialize = "postalCode"))]
    postal_code: String,
    #[serde(rename(deserialize = "requestedName"))]
    #[serde(rename(serialize = "requestedName"))]
    requested_name: String,
}

impl Purchase {
    pub fn new(name: String, promotion_code: String) -> Self {
        Self {
            product: Product::new("freetrial".to_string()),
            token: promotion_code,
            postal_code: "865304".to_string(),
            requested_name: name,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PurchaseRequest {
    purchase: Purchase
}

impl PurchaseRequest {
    pub fn new(purchase: Purchase) -> Self {
        Self {
            purchase,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PurchaseResponse {
    id: String,
    #[serde(rename(serialize = "publicUrl"))]
    #[serde(rename(deserialize = "publicUrl"))]
    public_url: String,
    #[serde(rename(deserialize = "secretUrl"))]
    #[serde(rename(serialize = "secretUrl"))]
    secret_url: String,
    #[serde(rename(deserialize = "hashId"))]
    #[serde(rename(serialize = "hashId"))]
    hash_id: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PurchaseCreds {
    ak: String,
    sk: String,
}

pub async fn check_name(name: String) -> bool {
    let url = format!("http://{}.weblum.photos", name);
    let headers = Headers::new();
    headers.append("Content-Type", "application/json");
    let request = Request::get(&url).headers(headers);
    match request.send().await {
        Ok(_) => false,
        Err(_) => true,
    }
}

pub async fn purchase_creds(site_name: String, promotion_code: String) -> Option<AWSCredentials> {
    let purchase = Purchase::new(site_name, promotion_code);
    let url = "https://pu1jqa2403.execute-api.us-east-1.amazonaws.com/test/weblum-api".to_string();
    
    let purchase_request = PurchaseRequest::new(purchase);

    let headers = Headers::new();
    headers.append("Content-Type", "application/json");

    let serialized = serde_json::to_string(&purchase_request).expect("should deserialize");

    match RequestBuilder::new(&url).method(Method::POST).headers(headers).body(&serialized) {
        Ok(request) => {
            match request.send().await {
                Ok(response) => {
                    match response.ok() {
                        true => {
                            let text = response.text().await.expect("text expected");
                            match serde_json::from_str::<PurchaseResponse>(&text) {
                                Ok(purchase_response) => {
                                    let parts = purchase_response.secret_url.split_once("=").expect("hash expected");
                                    let decoded = decode_string(parts.1.to_string());
                                    match serde_json::from_str::<PurchaseCreds>(&decoded.expect("decoded")) {
                                        Ok(creds) => {
                                            return Some(AWSCredentials::new(creds.ak, creds.sk));
                                        },
                                        Err(_) => return None,
                                    }
                                },
                                Err(_) => None,
                            }
                        },
                        false => {
                            return None;
                        }
                    }
                },
                Err(_) => {
                    log::info!("Error from purchase request");
                    None
                }
            }
        },
        Err(_) => {
            log::info!("Error building purchase request");
             None
        },
    }
}


#[component]
pub fn PurchaseControl(
) -> impl IntoView {

    let toasts = expect_context::<Toasts>();

    let (input_site_name, set_input_site_name ) = create_signal( "".to_string());

    let (purchased_site_name, set_purchased_site_name) = create_signal("".to_string());

    let (promotion_code, set_promotion_code ) = create_signal("".to_string());

    let (name_available, set_name_available) = create_signal(None::<String>);

    let default_creds: Option<AWSCredentials> = None;
    let (purchased_aws_credentials, set_purchased_aws_credentials) = create_signal(default_creds);

    let check_name_action = create_action( move |_: &String| async move {
        if !check_name(input_site_name.get_untracked()).await {
            set_name_available.set(Some("Name in use.".to_string()));
        } else {
            set_name_available.set(None);
        }
    });

    let (purchase_requested, set_purchase_requested) = create_signal(false);
    let (purchase_failed, set_purchase_failed) = create_signal(false);
    let (purchase_succeeded, set_purchase_succeeded) = create_signal(false);

    let purchase_site_action = create_action(move |_: &String| async move {
        set_purchase_requested.set(true);
        match purchase_creds(input_site_name.get_untracked(), promotion_code.get_untracked()).await {
            Some(creds) => {
                set_purchased_aws_credentials.set(Some(creds));
                set_purchase_requested.set(false);
                set_purchased_site_name.set(input_site_name.get_untracked());
                set_purchase_succeeded.set(true);
            },
            None => {
                set_purchase_failed.set(true);
                set_purchase_requested.set(false);
                set_purchase_succeeded.set(false);
            },
        }
    });

    //purchase complete listener
    create_effect(move |_| {
        match purchased_aws_credentials.get() {
            Some(creds) => {
                let conf = SiteConfig::new(format!("{}.weblum.photos", input_site_name.get_untracked()), Some(creds.access_key), Some(creds.secret_key), "us-west-2".to_string());
                match add_site_config(conf) {
                    Ok(site_config) => {
                        match set_current_site(site_config.id) {
                            Ok(current) => log::info!("Current site set to: {}", current.s3_bucket_name()),
                            Err(_) => log::info!("Error setting current site"),
                        }
                    },
                    Err(_) => log::info!("Error adding new site to config"),
                }
            },
            None => (),
        };
    });

    // purchase failed listener
    create_effect(move |_| {
        if purchase_failed.get() {
            toasts.push(
                Toast {
                    id: Uuid::new_v4(),
                    created_at: time::OffsetDateTime::now_utc(),
                    variant: ToastVariant::Error,
                    header: "Purchase Failed".into_view(),
                    body: "Is purchase code is valid?".into_view(),
                    timeout:  ToastTimeout::DefaultDelay,
                }
            )
        }
    });

    // requested site name change listener
     create_effect(move |_| {
        if input_site_name.get().len() > 3 {
            let _ = check_name_action.dispatch("click".to_string());
        } else {
            set_name_available.set(Some("Name must be 4 characters or more.".to_string()));
        }
    });

    view!{
        <div
            style = {match get_device_type() {
                    DeviceType::Mobile => "width: 310px",
                    DeviceType::Desktop => "",
                }}
        >
          <div>
            <h3>"Build Your Own Site"</h3>
            <label>"Site name"</label>
            <div style="display: flex; flex-direction: row;">
                <TextInput
                    get={input_site_name}
                    set={set_input_site_name}
                    placeholder="mysite"
                />
                <div>
                ".weblum.photos"
                </div>
            </div>

            <div style="display: flex; flex-direction: row; padding: 10px;">
                <div style="padding: 0 0 0 10px;">
                    {move || match name_available.get() {
                        None => view!{<Chip color=ChipColor::Success>"Available"</Chip>},
                        Some(msg) => view!{<Chip color=ChipColor::Warn>{msg}</Chip>},
                    }}
                </div>
            </div>

            <div>
                <div>
                    <div style="border: 1px solid grey; padding: 5px;">
                        "To receive a free purchase code, contact phil.kahrl at gmail"
                    </div>
                    <label>"Purchase Code"</label>

                </div>
                <TextInput
                    get={promotion_code}
                    set={set_promotion_code}
                    placeholder="purchase code"
                />
                <div style="padding: 10px">
                    <Button
                        disabled = Signal::derive(move || {promotion_code.get().len() < 4  || purchase_requested.get() || name_available.get().is_some() })
                        on_click=move |_evt| {
                            purchase_site_action.dispatch("click".to_string());
                        }
                    >
                        "Build Site"
                    </Button>
                </div>
               
                <div>
                    {move || {
                        if purchase_requested.get() {
                             view!{<div><Icon icon=leptos_icons::Icon::from(CgSpinner) /><div>"Waiting ..."</div></div>}.into_view()
                        } else {"".into_view()}
                    }}
                </div>

                {move || {
                    if purchase_succeeded.get() {
                        view!{
                            <div>
                                <div style="font-face: bold; font-size: 1.5em;">"Congratulations!"</div>

                                <div style="display: flex; flex-direction: row;">
                                    <div>"You are now the proud owner of the site at."</div>
                                    <a 
                                        style="padding-left: 10px;"
                                        target="_blank"
                                        href={format!("http://{}.weblum.photos", purchased_site_name.get())}
                                    >
                                        "Public Link"
                                    </a>
                                </div>

                                <div>"Use the 'Add Image' button to add photos to your site."</div>

                                <Button
                                    on_click=move |_evt| {
                                        let _ = leptos::window().location().reload();
                                    }
                                >
                                    "Ok"
                                </Button>


                            </div>
                        }.into_view()
                    } else {
                        "".into_view()
                    }
                }}
                <div style="border: 1px solid grey; padding: 5px; display: flex; flex-direction: row; justify-content: space-between;">
                    <a target = "_blank" href="https://weblum.photos/termsofuse.html">"Terms of Use. "</a>
                    <a target = "_blank" href="https://weblum.photos/privacypolicy.html">"Privacy Policy"</a>
                </div>
            </div>
          </div>
        </div>
    }
}