use bytes::Bytes;
use gloo_net::http::Request;

use hmac::{Hmac, Mac};

use secrecy::{ExposeSecret, SecretString};

use sha2::{Sha256, Digest};

use time::{Duration, OffsetDateTime};

use urlencoding::encode;
use url::{Url};
use gloo_net::http::Headers;
use js_sys::Date;

pub const USER_AGENT: &str = "wasm";
pub static ALGORITHM: &str = "AWS4-HMAC-SHA256";
static AWS4_REQUEST: &str = "aws4_request";
static SIGNED_HEADERS: &str = "content-type;host;x-amz-content-sha256;x-amz-date;x-amz-user-agent";

use time::{
    format_description::well_known::Iso8601,
    macros::{format_description, offset},
};

pub fn generate_headers(now: &Date) -> Headers {
    let headers = Headers::new();
    let date_string = amzdate(&date_time_from_js_sys_date(now.clone()));
    headers.append("X-Amz-Content-Sha256", "UNSIGNED-PAYLOAD");
    headers.append("X-Amz-Date", &date_string);
    headers.append("X-Amz-User-Agent", USER_AGENT);
    headers.append("Content-Type", "application/octet-stream");
    headers.append("Connection", "keep-alive");
    headers
}

pub fn amzdate(date: &OffsetDateTime) -> String {
    use time::format_description::well_known::iso8601::{Config, TimePrecision};
    // The date that is used to create the signature. The format must be ISO 8601 basic format (YYYYMMDD'T'HHMMSS'Z').
    // For example, the following date time is a valid X-Amz-Date value: 20120325T120000Z.
    // https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/CommonParameters.html
    const AMZCONF: u128 = Config::DEFAULT
        .set_use_separators(false)
        .set_time_precision(TimePrecision::Second {
            decimal_digits: None,
        })
        .encode();

    date
        .to_offset(offset!(UTC))
        .format(&Iso8601::<AMZCONF>)
        .unwrap()
}

fn sign(key: &[u8], message: String) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(&message.into_bytes());
    mac.finalize().into_bytes()[..].to_vec()
}

fn datestamp(date: &OffsetDateTime) -> String {
    date
        .to_offset(offset!(UTC))
        .format(format_description!("[year][month][day]"))
        .unwrap()
}

//DateKey              = HMAC-SHA256("AWS4"+"<SecretAccessKey>", "<YYYYMMDD>")
//DateRegionKey        = HMAC-SHA256(<DateKey>, "<aws-region>")
//DateRegionServiceKey = HMAC-SHA256(<DateRegionKey>, "<aws-service>")
//SigningKey           = HMAC-SHA256(<DateRegionServiceKey>, "aws4_request")

fn signature_key(
    secret_access_key: &SecretString,
    region: &str,
    date: &OffsetDateTime,
    service: &str,
) -> Vec<u8> {
    let k_date = sign(
        &format!("AWS4{}", secret_access_key.expose_secret()).into_bytes(),
        datestamp(date),
    );
    let k_region = sign(k_date.as_slice(), region.to_string());
    let k_service = sign(k_region.as_slice(), service.to_string());
    sign(k_service.as_slice(), AWS4_REQUEST.to_string())
}


pub fn encoded_hash_from_string(message: impl ToString) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message.to_string().into_bytes());
    hex::encode(hasher.finalize())
}

pub fn encoded_hash_from_byte_vec(message: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hex::encode(hasher.finalize())
}

pub fn _hash_bytes_to_hexstring(message: &Bytes) -> String {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hex::encode(hasher.finalize())
}

fn credential_scope(region: &str, date: &OffsetDateTime, service: &str) -> String {
    format!(
        "{}/{}/{}/{}",
        datestamp(date),
        region,
        service,
        AWS4_REQUEST
    )
}

async fn _get_payload_hash(request: &Request) -> Option<String> {
    match request.binary().await {
        Ok(file_data) => {
            log::info!("We have a binary");
            let bytes = Bytes::copy_from_slice(&file_data);
            let hashed_payload = _hash_bytes_to_hexstring(&bytes);
            Some(hashed_payload)
        },
        Err(_) => None
    }
}
//https://docs.aws.amazon.com/AmazonS3/latest/API/sig-v4-header-based-auth.html
//<HTTPMethod>\n
//<CanonicalURI>\n
//<CanonicalQueryString>\n
//<CanonicalHeaders>\n
//<SignedHeaders>\n
//<HashedPayload>

fn canonicalize_headers(request: Request) -> Option<String> {
    let allowed_headers_keys: Vec<&str> = SIGNED_HEADERS.rsplit(';').collect();
    let mut values_and_keys: Vec<String> = vec![];
    for entry in request.headers().entries() {
        if allowed_headers_keys.contains(&entry.0.to_lowercase().trim()) {
            values_and_keys.push(format!("{}:{}", entry.0, entry.1));
        }
    }
    let parse_result = Url::parse(&request.url()).expect("Unable to parse url");
    values_and_keys.push(format!("{}:{}", "host", parse_result.host().expect("host expected")));
    values_and_keys.sort();
    Some(format!("{}{}{}", values_and_keys.join("\n"), '\n', '\n'))
}

pub async fn canonicalize_request(request: Request) -> Option<String> {
    let payload_hash = "UNSIGNED-PAYLOAD";
    //let payload_hash = get_payload_hash(&request).await.expect("payload hash expected");
    let http_method = format!("{}{}", request.method().as_str(), '\n');
    let parse_result = Url::parse(&request.url()).expect("Unable to parse url");
    let canonical_uri = format!("{}{}", parse_result.path(), '\n');
    log::info!("{}", canonical_uri);
    let query_string = encode(parse_result.query().unwrap_or(""));
    let canonical_query_string = format!("{}{}", query_string, '\n');
    let canonical_headers = canonicalize_headers(request).expect("canonical headers expected");
    let signed_headers = format!("{}{}", SIGNED_HEADERS.to_lowercase(), '\n');
    let result = Some(format!("{}{}{}{}{}{}", http_method, canonical_uri, canonical_query_string, canonical_headers, signed_headers, payload_hash));
    log::info!("canonicalized request: {}", &result.clone().unwrap());
    result
}

fn date_from_epoch(epoch: i64) -> OffsetDateTime {
    let duration = Duration::new(epoch, 0);
    let epoch_time = OffsetDateTime::UNIX_EPOCH;
    let date = epoch_time.checked_add(duration).expect("unable to create time");
    date
}

fn date_time_from_js_sys_date(js_sys_date: Date) -> OffsetDateTime {
    date_from_epoch((js_sys_date.get_time()/1000.0) as i64)
}

pub async fn get_auth_header (
    secret_access_key: &SecretString,
    access_key: String,
    region: &str,
    request: Request,
    epoch: i64, 
) -> String {

    let date = date_from_epoch(epoch);
    let credential_scope = credential_scope(region, &date, "s3");
    let canonical_request = canonicalize_request(request).await.expect("canonicalized request expected");
    log::info!("Canonical request:");
    log::info!("{}", canonical_request);
    let encoded_canonical_request = encoded_hash_from_string(canonical_request);

    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        ALGORITHM,
        amzdate(&date),
        credential_scope,
        encoded_canonical_request,
    );

    log::info!("string to sign:");
    log::info!("{}", &string_to_sign);

    let key = signature_key(secret_access_key, region, &date, "s3");
    let request_signature = hex::encode(sign(key.as_slice(), string_to_sign));
    //let request_signature = encoded_hash_from_byte_vec(sign(key.as_slice(), string_to_sign));
    let formatted_date = amzdate(&date);
    let (ymd, _) = formatted_date.split_at(8);

    format!(
        "AWS4-HMAC-SHA256 Credential={}/{}/{}/s3/aws4_request,SignedHeaders={},Signature={}",
        access_key,
        ymd,
        region,
        SIGNED_HEADERS, 
        request_signature,
    )
}

#[cfg(test)]
mod tests {
    use secrecy::SecretString;
    use time::macros::datetime;
    use crate::awssigv4::{amzdate, signature_key, sign};
    use std::str::FromStr;
    use gloo_net::http::RequestBuilder;
    use gloo_net::http::Method;
    use js_sys::Date;

    use crate::awssigv4::generate_headers;
    use crate::awssigv4::credential_scope;
    use crate::awssigv4::date_from_epoch;
    use crate::awssigv4::datestamp;
    use crate::awssigv4::encoded_hash_from_string;

    #[test]
    fn test_datestamp() {
        let date = date_from_epoch(1697123242);
        let result = datestamp(&date);
        assert_eq!(result, "20231012");
    }

    #[test]
    fn test_credential_scope() {
        let date = date_from_epoch(1697123242);
        let credential_scope = credential_scope("us-west-2", &date, "s3");
        assert_eq!(credential_scope, "20231012/us-west-2/s3/aws4_request");
    }

    #[test]
    fn test_amzdate() {
        let sign_time = datetime!(2015-08-30 12:36:00 UTC);
        let result = amzdate(&sign_time);
        let expected = "20150830T123600Z";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_signature() {
        let secret_string = SecretString::from_str("wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY").expect("secret expected");
        let region = "us-west-2";
        let service = "s3";
        let payload = "payloadstring";
        //let result = signature(&secret_string, region, service, payload, 0);
        //let expected = "ce912658a7f832819fcec0acd47dd1bd343e34105832192024e9432c7887091a";
        //assert_eq!(result, expected);
        
    }

    #[test]
    fn test_signature_key() {
        let message = r#"AWS4-HMAC-SHA256
20150830T123600Z
20150830/us-east-1/iam/aws4_request
f536975d06c0309214f805bb90ccff089219ecd68b2577efef23edd43b7e1a59"#;

        let secret_access_key = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".parse().unwrap();
        let sign_time = datetime!(2015-08-30 12:36:00 UTC).into();

        let key = signature_key(&secret_access_key, "us-east-1", &sign_time, "iam");

        let expected = "5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7";
        let actual = hex::encode(sign(key.as_slice(), message.to_string()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_hash() {
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let actual = encoded_hash_from_string("");
        assert_eq!(actual, expected);
    }
}