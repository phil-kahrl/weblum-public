use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use serde::*;
use thiserror::Error;
use uuid::Uuid;
use hex::FromHexError;
use std::num::TryFromIntError;
use gloo_storage::errors::StorageError;
use wasm_bindgen::JsValue;

const SITE_CONFIG: &str = "WM_SITE_CONFIG";
const CURRENT_SITE: &str = "WM_CURRENT_SITE";

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("io")]
    Io,
    #[error("serialization")]
    Serialization,
    #[error("encoding")]
    Encoding,
    #[error("base64")]
    Base64,
    #[error("site not found")]
    SiteNotFound,
    #[error("unable to read from local storage")]
    Storage,
    #[error("other")]
    Other,
}

impl From<String> for ConfigError {
    fn from(_e: String) -> Self {
        Self::Io
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(_e: serde_json::Error) -> Self {
        Self::Serialization
    }
}

impl From<StorageError> for ConfigError {
    fn from(_e: StorageError) -> Self {
        Self::Storage
    }
}

impl From<JsValue> for ConfigError {
    fn from(_e: JsValue) -> Self {
        Self::Other
    }
}

impl From<std::string::FromUtf8Error> for ConfigError {
    fn from(_e: std::string::FromUtf8Error) -> Self {
        Self::Encoding
    }
}

impl From<FromHexError> for ConfigError {
    fn from(_e: FromHexError) -> Self {
        Self::Base64
    }
}

impl From<TryFromIntError> for ConfigError {
    fn from(_e: TryFromIntError) -> Self {
        Self::Other
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SiteConfig {
    pub id: String,
    #[serde(rename(deserialize = "b"))]
    #[serde(rename(serialize = "b"))]
    pub _s3_bucket_name: String,
    // these are optional, but app will be read-only
    #[serde(rename(deserialize = "a"))]
    #[serde(rename(serialize = "a"))]
    pub access_key: Option<String>,
    #[serde(rename(deserialize = "s"))]
    #[serde(rename(serialize = "s"))]
    pub secret_key: Option<String>,
    #[serde(rename(deserialize = "r"))]
    #[serde(rename(serialize = "r"))]
    pub region: String,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub read_only: bool,
}

impl SiteConfig {
    pub fn new (
        s3_bucket_name: String,
        access_key: Option<String>,
        secret_key: Option<String>,
        region: String,
    ) -> Self {

        let ak = match access_key {
            Some(_a) => Some(_a),
            None => None,
        };

        let sk = match secret_key {
            Some(_s) => {
                Some(_s)
            },
            None => None,
        };

        Self {
            id: String::from(Uuid::new_v4().to_string()),
            _s3_bucket_name: s3_bucket_name,
            access_key: ak,
            secret_key: sk,
            region,
            read_only: true,
        }
    }

    pub fn s3_bucket_name(&self) -> String {
        self._s3_bucket_name.to_lowercase()
    }

    pub fn encoded(&self) -> String {
       hex::encode(serde_json::to_string(self).expect("should deserialize"))
    }

    pub fn from_encoded (
        encoded: String,
    ) -> Result<Self, ConfigError> {
        match serde_json::from_str::<SiteConfig>(&encoded) {
            Ok(sc) => {
                return Ok(Self::new(sc._s3_bucket_name, sc.access_key, sc.secret_key, sc.region))
            },
            Err(_err) => {
                let decoded1 = hex::decode(encoded)?;
                let decoded = String::from_utf8(decoded1)?;
                let deserialized =  serde_json::from_str::<SiteConfig>(&decoded)?;
                Ok(Self::new(deserialized.s3_bucket_name(), deserialized.access_key, deserialized.secret_key, deserialized.region))
            },
        }
    }
}

fn bucket_name_from_url() -> Option<String> {
    let loc = leptos::window().location().host().expect("location host expected");
    match loc.rfind(".weblum.photos") {
        Some(_) => {
            let port_removed = loc.rsplit(":").collect::<Vec<_>>()[0];
            Some(port_removed.to_string())
        },
        None => None
    }
}

pub fn get_sites() -> Result<Vec<SiteConfig>, ConfigError> {
    match LocalStorage::get::<String>(SITE_CONFIG) {
        Ok(result) => {
            let decoded = String::from_utf8(hex::decode(result.clone())?)?;
            Ok(serde_json::from_str::<Vec<SiteConfig>>(&decoded)?)
        },
        Err(_err) => {
            Ok(vec![])
        }
    }
}

pub fn add_site_config(site_config: SiteConfig) -> Result<SiteConfig, ConfigError> {
    let mut sites = get_sites()?;
    sites.push(site_config.clone());
    let _ = LocalStorage::set(SITE_CONFIG, hex::encode(serde_json::to_string(&sites)?));
    Ok(site_config.clone())
}

pub trait SingleSiteRuntimeConfig {
    fn get_current_config() -> Result<SiteConfig, ConfigError>;
}

pub trait MultiSiteRuntimeConfig {
    fn set_current_site(&self, site_name: String) -> Result<SiteConfig, ConfigError>;
    fn remove_site(&self, id: String) -> Result<(), ConfigError>;
    fn get_sites(&self) -> Result<Vec<SiteConfig>, ConfigError>;
    fn add_site_config(&self, site_config: SiteConfig) -> Result<SiteConfig, ConfigError>;
}

pub struct HashRouteRuntimeConfig {

}

impl SingleSiteRuntimeConfig for HashRouteRuntimeConfig {
    fn get_current_config() -> Result<SiteConfig, ConfigError> {
        let mut raw = leptos::window().location().hash()?;
        match raw.len() > 20 {
            true => {
                let hash = raw.split_off(1);
                let config = SiteConfig::from_encoded(hash)?;
                Ok(config)
            },
            false => {
                Err("hash route insufficient".to_string().into())
            }
        }  
    }
}

pub struct LocalStorageRuntimeConfig {

}

impl SingleSiteRuntimeConfig for LocalStorageRuntimeConfig {
    fn get_current_config() -> Result<SiteConfig, ConfigError> {
        let current_site = LocalStorage::get::<String>(CURRENT_SITE)?;
        let sites = get_sites()?;
        if sites.len() > 0 {
            for site in &sites {
                if site.id == current_site {
                    return Ok(site.clone())
                }
            }
            return Ok(sites.get(0).expect("element").clone())
        } else {
            Err(ConfigError::SiteNotFound)          
        }
    }
}

pub fn set_current_site(site_name: String) -> Result<SiteConfig, ConfigError> {
    let sites = get_sites()?;
    for site in sites {
        if site.id == site_name {
            let _ = LocalStorage::set(CURRENT_SITE, site_name);
            return Ok(site)
        }
    };
    Err(ConfigError::SiteNotFound)
}

pub fn remove_site(id: String) -> Result<(), ConfigError> {
    let mut sites = get_sites()?;
    let mut to_remove: i16 = -1;
    for (i, site) in sites.iter().enumerate() {
        if site.id == id {
            to_remove = i as i16;
            break;
        }
    }
    if to_remove > -1 {
        sites.remove(to_remove.try_into()?);
        let _ = LocalStorage::set(SITE_CONFIG, hex::encode(serde_json::to_string(&sites)?));
    };
    Ok(())
}

pub fn get_current_config() -> Result<SiteConfig, ConfigError> {
    match HashRouteRuntimeConfig::get_current_config() {
        Err(_e) => {
            log::info!("loading config from local storage");
            match LocalStorage::get::<String>(CURRENT_SITE) {
                Err(_e) => {
                    match bucket_name_from_url() {
                        Some(name) => {
                            let _ = add_site_config(SiteConfig::new(name.clone(), None, None, "us-west-2".to_string()));
                            Ok(SiteConfig::new(name, None, None, "us-west-2".to_string()))
                        },
                        None =>  Ok(SiteConfig::new("app.weblum.photos".to_string(), None, None, "us-west-2".to_string()))
                    }
                },
                Ok(current_site) => {
                    let sites = get_sites()?;
                    if sites.len() > 0 {
                        for site in &sites {
                            if site.id == current_site {
                                return Ok(site.clone())
                            }
                        }
                        return Ok(sites.get(0).expect("element").clone())
                    } else {          
                        match bucket_name_from_url() {
                            Some(name) => {
                                let _ = add_site_config(SiteConfig::new(name.clone(), None, None, "us-west-2".to_string()));
                                Ok(SiteConfig::new(name, None, None, "us-west-2".to_string()))
                            },
                            None =>  Ok(SiteConfig::new("app.weblum.photos".to_string(), None, None, "us-west-2".to_string()))
                        }
                    }
                }
            }
        },
        Ok(config) => {
            log::info!("loading config from hash route");
            return Ok(config);
        }
    }
}

