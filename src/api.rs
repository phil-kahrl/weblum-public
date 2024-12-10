use leptos::*;

use thiserror::Error;
use serde::{Serialize, Deserialize};

use gloo_net::http::{Request};
use js_sys::{Date, Uint8Array};

use crate::SiteSettings;
use wasm_bindgen::JsValue;
use crate::ListBucketResult;
use crate::S3ObjectInfo;
use crate::ImageInfo;

use crate::api::Error::UploadError;
use crate::api::Error::Fetch;

use url::form_urlencoded::byte_serialize;
use crate::local_config::get_current_config;
use crate::awssigv4::generate_headers;
use crate::upload_to_s3;
use crate::delete_object;
use crate::get_image;
use crate::local_config;

#[derive(Serialize, Deserialize, Clone)]
pub struct AWSCredentials {
    pub access_key: String,
    pub secret_key: String,
}

impl AWSCredentials {
    pub fn new(ak: String, sk: String) -> Self {
        Self {
            access_key: ak,
            secret_key: sk,
        }
    }
}

pub fn bucket_name() -> String {
    match get_current_config() {
        Ok(config) => {
            config.s3_bucket_name()
        },
        Err(err) => {
            log::info!("Err {:?}", err);
            "".to_string()
        },
    }
}

async fn send_list_request(url: String) -> Result<ListBucketResult> {
    let request = Request::get(&url).headers(generate_headers(&Date::new_0()));
    return match request.send().await {
        Ok(response) => {
            let xml = response.text().await.expect("xml expected");
            match quick_xml::de::from_str(&xml) {
                Ok(result) => Ok(result),
                Err(err) => {
                    log::info!("Error on deserializing request: {}", err);
                    Ok(ListBucketResult::new())
                },
            }
        },
        Err(net_error) => {
            Err(Fetch(format!("{}", net_error).to_string()))
        }
    };
}

pub async fn list_images() -> Result<Vec<S3ObjectInfo>> {
    let mut contents: Vec<S3ObjectInfo> = vec![];
    let config = get_current_config()?;
    let bucket = config.s3_bucket_name();
    let region = config.region;
    let url = {format!("https://s3.{}.amazonaws.com/{}?list-type=2&prefix=images%2F&start-after=images%2F", region, bucket)};
    let mut response = send_list_request(url).await?;
    contents.append(&mut response.contents);
    let mut truncated = response.is_truncated;
    let mut next_continuation_token = response.next_continuation_token;
    while truncated {
        match next_continuation_token {
            Some(ref continuation_token) => {
                let url = format!("https://s3.{}.amazonaws.com/{}?list-type=2&continuation-token={}&prefix=images%2F&start-after=images%2F",
                    region, bucket_name(), byte_serialize(continuation_token.as_bytes()).collect::<String>());
                let mut r = send_list_request(url).await?;
                truncated = r.is_truncated;
                next_continuation_token = r.next_continuation_token;
                contents.append(&mut r.contents);
            },
            None => truncated = false,
        } 
    }
    contents.sort_by(|a, b| b.last_modified().cmp(&a.last_modified()));     
    Ok(contents)
}

pub fn region() -> String {
    get_current_config().expect("config expected").region
}

pub async fn set_public_site_settings(site_settings: SiteSettings) -> Result<SiteSettings> {
    let s3_bucket_name = get_current_config().expect("config expected").s3_bucket_name();
    let region = get_current_config().expect("config expected").region;
    let sk = get_current_config().expect("config expected").secret_key.expect("sk");
    let ak = get_current_config().expect("config expected").access_key.expect("ak");

    let source = serde_json::to_string(&site_settings).expect("site setting expected");
    let result = upload_to_s3(ak, sk, region, s3_bucket_name, "admin/settings.json".to_string(), source.into()).await;
    let converted = result.as_string().expect("upload result exists");
    if converted == "error" {
       Err(UploadError("".to_string()))
    } else {
        Ok(site_settings)
    }
}

pub fn aws_credentials() -> Option<AWSCredentials> {
    let access_key = match get_current_config().expect("ak").access_key {
        Some(ak) => {
            Some(ak)
        },
        None => None,
    };
    let secret_key = match get_current_config().expect("sk").secret_key {
        Some(sk) => {
            Some(sk)
        },
        None => None,
    };
    Some(AWSCredentials::new(access_key.expect("ak"), secret_key.expect("sk")))
}

pub async fn upload_image_1(source: JsValue, key: String) -> Result<String> {
    let creds = aws_credentials().expect("credential expected");
    log::info!("Attempting upload");
    let result = upload_to_s3(creds.access_key, creds.secret_key, region(), bucket_name(), key, source).await;
    log::info!("Upload complete");
    let converted = result.as_string().expect("upload result exists");
    log::info!("converted {}", converted);
    if converted == "error" {
        Err(UploadError("".to_string()))
    } else {
        Ok("ok".to_string())
    }
}

pub async fn upload_object(source: JsValue, _prefix: String, object_name: String) -> bool {
    let s3_bucket_name = get_current_config().expect("config expected").s3_bucket_name();
    let region = get_current_config().expect("config expected").region;
    let secret_key = get_current_config().expect("config expected").secret_key.expect("sk");
    let access_key = get_current_config().expect("config expected").access_key.expect("ak");
    let result = upload_to_s3(access_key, secret_key, region, s3_bucket_name, object_name, source).await;
    let converted = result.as_string().expect("upload result exists");
    if converted == "error" {
        false
    } else {
        true
    }
}

pub async fn rename_image(old_filename: String, new_filename: String, set_error: RwSignal<Option<String>>) {
    let current_image = get_image(old_filename.clone()).await;
    match current_image {
        Some(ci) => {
            let u = Uint8Array::new_with_length(ci.len() as u32);
            u.copy_from(&ci);
            let _delete_result = delete_object(format!("images/{}", old_filename), set_error).await;
            let _upload_result = upload_object(u.into(), "".to_string(), format!("images/{}", new_filename)).await;
        },
        None => ()
    }
}

pub async fn update_comment(comment_text: String, e_tag: String, set_error: RwSignal<Option<String>>) {
    set_error.set(None);
    let s3_bucket_name = get_current_config().expect("config expected").s3_bucket_name();
    let region = get_current_config().expect("config expected").region;
    let secret_key = get_current_config().expect("config expected").secret_key.expect("sk");
    let access_key = get_current_config().expect("config expected").access_key.expect("ak");
    let key = format!("comments/{}", e_tag);
    let result = upload_to_s3(access_key, secret_key, region, s3_bucket_name, key, comment_text.into()).await;
    let converted = result.as_string().expect("result exists");
    if converted == "error" {
        set_error.set(Some("error on update comment".to_string()));
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Serialize, Deserialize, Clone)]
pub enum Error {
    #[error("Failed to fetch")]
    Fetch(String),
    #[error("Upload Error")]
    UploadError(String),
    #[error("Local Config Error")]
    LocalConfigError,
}

impl From<local_config::ConfigError> for Error {
    fn from(_e: local_config::ConfigError) -> Self {
        Self::LocalConfigError
    }
}

