use leptos::*;

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use js_sys::Date;
use gloo_net::http::Request;

use base64::*;
use base64::alphabet::Alphabet;
use base64::alphabet::ParseAlphabetError;

use wasm_bindgen::prelude::wasm_bindgen;
use crate::api::*;
use crate::local_config::get_current_config;
use crate::file_metadata_reader::*;
use crate::api::list_images;
use crate::awssigv4::*;

mod api;
mod hash_route;
mod components;
mod awssigv4;
mod file_metadata_reader;
mod local_config;

use self::{components::*};

const MOBILE: &str = "Mobile";

pub enum DeviceType {
    Desktop,
    Mobile,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SiteSettings {
    #[serde(rename(deserialize = "appTitle"))]
    #[serde(rename(serialize = "appTitle"))]
    app_title: String,
    #[serde(rename(deserialize = "pageTitle"))]
    #[serde(rename(serialize = "pageTitle"))]
    page_title: String,
}

impl SiteSettings {
    pub fn new() -> Self {
        Self {
            app_title: "".to_string(),
            page_title: "".to_string(),
        }
    }
}


/**
 * Base64 implementation
 */ 
const CUSTOM_ALPHABET: Result<Alphabet, ParseAlphabetError> = 
    base64::alphabet::Alphabet::new("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");

pub fn encode_binary(data: Vec<u8>) -> String {
    let engine = base64::engine::GeneralPurpose::new(
        &CUSTOM_ALPHABET.unwrap(),
        base64::engine::general_purpose::PAD);
    engine.encode(data)
}

pub fn decode_string(data: String) -> Option<String> {
    let engine = base64::engine::GeneralPurpose::new(
        &CUSTOM_ALPHABET.unwrap(),
        base64::engine::general_purpose::PAD);
    match engine.decode(&String::from(&data)) {
        Ok(bytes) => Some(String::from_utf8(bytes).expect("utf8 decodes")),
        Err(_) => None
    }
}


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = uploadToS3)]
    pub async fn upload_to_s3(ak: String, sk: String, region: String, bucket_name: String, key: String, source: JsValue) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = deleteObject)]
    pub async fn extern_delete_object(ak: String, sk: String, region: String, bucket_name: String, prefix: String, filename: String) -> JsValue;
}

pub async fn get_public_site_settings() -> SiteSettings {
    match get_current_config() {
        Ok(config) => {
            let url = format!("https://s3.{}.amazonaws.com/{}/admin/settings.json", config.region, config.s3_bucket_name());
            let request = Request::get(&url).headers(generate_headers(&Date::new_0()));
            match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        let text_result = response.text().await;
                        match text_result {
                            Ok(s) => {
                                match serde_json::from_str::<SiteSettings>(&s) {
                                    Ok(settings) => settings,
                                    Err(_) => SiteSettings::new(),
                                }
                            },
                            Err(_) => SiteSettings::new(),
                        }
                    } else {
                        SiteSettings::new()
                    }
                },
                Err(_) => SiteSettings::new(),
            }
        },
        Err(_) => SiteSettings::new(),
    }
}

pub async fn delete_object(filename: String, set_error: RwSignal<Option<String>> ) -> bool {
    set_error.set(None);
    match get_current_config() {
        Ok(current_config) => {
            let s3_bucket_name = current_config.s3_bucket_name();
            let access_key = current_config.access_key.expect("access key");
            let secret_key = current_config.secret_key.expect("secret key");
            let region = current_config.region;

            let delete_result = extern_delete_object(access_key, secret_key, region, s3_bucket_name, "".to_string(), filename).await;
            let converted = delete_result.as_string().expect("delete result exists");
            if converted == "error" {
                set_error.set(Some("Error while attempting to delete object".to_string()));
                false
            } else {
                true
            }
        },
        Err(_) => false,
    }
}

pub fn get_device_type() -> DeviceType {
    match leptos::window().navigator().user_agent() {
        Ok(s) => {
            if s.as_str().contains(MOBILE) {DeviceType::Mobile} else {DeviceType::Desktop}
        }
        Err(_) => DeviceType::Desktop,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
}

#[derive(Clone, Deserialize)]
pub struct AppState {
    current_image_index: usize,
    _current_image: String,
    _e_tag: String,
    current_caption: Option<String>,
}

impl AppState {
    pub fn new(contents: impl ImageInfo) -> Self {
        Self {
            _current_image: contents.key(),
            _e_tag: contents.e_tag(),
            current_caption: None,
            current_image_index: 0,
        }
    }

    pub fn current_caption(&self) -> Option<String> {
        self.current_caption.clone()
    }

    pub fn e_tag(&self, image_list: Vec<S3ObjectInfo>) -> String {
        match image_list.get(self.current_image_index) {
            Some(c) => c.e_tag().clone(),
            None => "none".to_string(),
        }
    }

    pub fn current_image_name(&self, image_list: Vec<S3ObjectInfo>) -> String {
        match image_list.get(self.current_image_index) {
            Some(c) => c.key().clone(),
            None => "none".to_string(),
        }
    }

     pub fn current_image_display_name(&self, image_list: Vec<S3ObjectInfo>) -> String {
        match image_list.get(self.current_image_index) {
            Some(c) => c.key().clone().replace("images/", ""),
            None => "none".to_string(),
        }
    }

    pub fn set_caption(&mut self, new_caption: String) {
        self.current_caption = Some(new_caption);
    }

    pub fn empty() -> Self {
        Self {
            _current_image: "images/none".to_string(),
            _e_tag: "none".to_string(),
            current_caption: None,
            current_image_index: 0,
        }
    }

    pub fn set_current_image(&mut self, key: String, image_list: Vec<S3ObjectInfo>) -> Self {
        for (i, e) in image_list.iter().enumerate() {
            if e.key() == key {
                self.current_image_index = i;
                break;
            }
        } 
        self.clone()  
    }

    pub fn previous_image(&mut self) -> Self {
        if self.current_image_index > 0 {
            self.current_image_index = self.current_image_index - 1
        }
        self.clone()
    }

    pub fn has_previous_image(&self) -> bool {
        self.current_image_index > 0
    }

    pub fn next_image(&mut self, image_list: Vec<S3ObjectInfo>) -> Self {
        if self.current_image_index < (image_list.len() - 1) {
            self.current_image_index = self.current_image_index + 1
        }
        self.clone()
    }

    pub fn has_next_image(&self, image_list: Vec<S3ObjectInfo>) -> bool {
        self.current_image_index < (image_list.len() - 1)
    }
}

fn default_contents() -> Vec<S3ObjectInfo> {
    vec![]
}

#[derive(Clone, Deserialize)]
pub struct ListBucketResult {
    #[serde(rename(deserialize = "NextContinuationToken"))]
    next_continuation_token: Option<String>,
    #[serde(rename(deserialize = "IsTruncated"))]
    is_truncated: bool,
    #[serde(rename(deserialize = "Contents"), default = "default_contents")]
    contents: Vec<S3ObjectInfo>,
}

impl ListBucketResult {
    pub fn new() -> Self {
        Self { 
            next_continuation_token: None,
            is_truncated: false,
            contents: vec![],
        }
    }
}


pub trait ImageInfo {
    fn key(&self) -> String;
    fn last_modified(&self) -> String;
    fn e_tag(&self) -> String;
    fn get_text(&self) -> Option<String>;
    fn set_text(&mut self, new_text: Option<String>);
}

#[derive(Clone, Deserialize, Serialize, Ord, Eq, PartialOrd, PartialEq)]
pub struct S3ObjectInfo {
    #[serde(rename(deserialize = "Key"))]
    _key: String,
    #[serde(rename(deserialize = "LastModified"))]
    _last_modified: String,
    #[serde(rename(deserialize = "ETag"))]
    _e_tag: String,
    #[serde(rename(deserialize = "Size"))]
    _size: i64,
    #[serde(rename(deserialize = "StorageClass"))]
    _storage_class: String,
    _text: Option<String>,
}

impl ImageInfo for S3ObjectInfo {
    fn key(&self) -> String {
        self._key.clone()
    }

    fn last_modified(&self) -> String {
        self._last_modified.clone()
    }

    fn e_tag(&self) -> String {
        self._e_tag.clone()
    }

    fn get_text(&self) -> Option<String> {
        self._text.clone()
    }

    fn set_text(&mut self, new_text: Option<String>) {
        self._text = new_text
    }

}

pub async fn get_image(filename: String) -> Option<Vec<u8>> {
    let region = get_current_config().expect("").region;
    let bucket = get_current_config().expect("").s3_bucket_name();
    let url = format!("https://s3.{}.amazonaws.com/{}/images/{}", region, bucket, filename);
    let request = Request::get(&url).headers(generate_headers(&Date::new_0()));
    match request.send().await {
        Ok(response) => {
            if response.ok() {
                return match response.binary().await {
                    Ok(s) => Some(s),
                    _ => None,
                }
            } else {
                log::info!("Request for image failed.");
                None
            }
        },
        Err(err) => {
            log::info!("Error on get image. {}", err);
            return None
        }
    } 
}

pub async fn get_comment(id: String) -> Option<String> {
    let region = get_current_config().expect("").region;
    let bucket = get_current_config().expect("").s3_bucket_name();
    let url = format!("https://s3.{}.amazonaws.com/{}/comments/{}", region, bucket, id);
    let request = Request::get(&url).headers(generate_headers(&Date::new_0()));
    let response = request.send().await.expect("response");
    if response.ok() {
        let text_result = response.text().await;
        match text_result {
            Ok(s) => Some(s),
            _ => Some("".to_string()),
        }
    } else {
        Some("".to_string())
    }
}

fn get_update_caption_action(app_state: RwSignal<Option<AppState>>, image_list: Vec<S3ObjectInfo>) -> Action<(), ()> {
    let (read_image_list, _) = create_signal(image_list);
    create_action(move |_| async move {
        let ut = app_state.get_untracked();
        match get_comment(ut.expect("").e_tag(read_image_list.get_untracked())).await {
            Some(contents) => {
                let mut current_state= app_state.get_untracked().expect("app state expected");
                current_state.set_caption(contents);
                app_state.set(Some(current_state));
            },
            _ => (),
        }
    })
}

pub fn update_app_state(app_state_signal: RwSignal<Option<AppState>>, new_app_state: AppState, image_list: Vec<S3ObjectInfo>) {
    app_state_signal.set(Some(new_app_state));
    get_update_caption_action(app_state_signal, image_list).dispatch(());
}

#[component]
pub fn App() -> impl IntoView {
    let app_state_signal = create_rw_signal(None::<AppState>);
    let (refetch_list_signal, set_refetch_list_signal) = create_signal(false);
    let (app_title, set_app_title) = create_signal("".to_string());

    let (public_site_settings, set_public_site_settings) = create_signal(SiteSettings::new());

    let fetch_public_site_settings = create_action(move |_: &String| async move {
        let s = get_public_site_settings().await;
        match leptos::window().document() {
            Some(doc) => {
                doc.set_title(&s.page_title);
                set_public_site_settings.set(s);
            },
            None => (), 
        }
    });
    
    let image_list_default: Option<Result<Vec<S3ObjectInfo>, api::Error>> = None;
    
    let (image_list, set_image_list) = create_signal(image_list_default);

    let fetch_images = create_action(move |_: &String| async move {
        let fetch_result = list_images().await;
        set_image_list.set(Some(fetch_result.clone()));
        match fetch_result {
            Ok(result) => {
                match app_state_signal.get_untracked() {
                    Some(app_state) => {
                        update_app_state(app_state_signal, app_state, result.clone());
                    },
                    None => {
                        let app_state = AppState::empty();
                        update_app_state(app_state_signal, app_state, result.clone());

                    }
                }
            },
            Err(_) => (),
        }
    });

    create_effect(move |_| match refetch_list_signal.get() {
        true => fetch_images.dispatch("refetch".to_string()),
        false => (),
    });

    // app state signal listener
    create_effect(move |_| {
        match  app_state_signal.get() {
            Some(_) => {
                //update_hash_route(Some(a)),
            },
            None => ()
        }
    });

    // public settings listener
    create_effect(move |_| {
        let settings = public_site_settings.get();
        set_public_site_settings.set(settings);
        set_app_title.set(public_site_settings.get_untracked().app_title);
    });

    fetch_public_site_settings.dispatch("".to_string());

    let (current_image, set_current_image) = create_signal(None::<String>);
    
    view!{
            <Home
                list_image_resource={image_list}
                public_settings={set_public_site_settings}
                app_title={app_title}
                set_refetch_list_signal = {set_refetch_list_signal} 
                app_state_signal = app_state_signal.into()
                current_image = {current_image}
                set_current_image = {set_current_image}
            />
    }

}


#[cfg(test)]
mod tests {
    use crate::decode_string;
    #[test]
    fn decode_base64() {

    }
}
