use leptos::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use js_sys::Date;
use gloo_net::http::RequestBuilder;
use gloo_net::http::Method;

use crate::awssigv4::generate_headers;
use crate::awssigv4::_canonicalize_request;


#[path="../src/awssigv4.rs"]
mod awssigv4;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_something() {
    let runtime = create_runtime();
    assert_eq!(1, 1);
    runtime.dispose();
}

#[wasm_bindgen_test]
fn test_generate_headers() {
    let result = generate_headers(&Date::new(&JsValue::from(1697122089958.0)));
    assert_eq!(result.get("X-Amz-Acl").unwrap(), "public-read");
    assert_eq!(result.get("X-Amz-Content-Sha256").unwrap(), "UNSIGNED-PAYLOAD");
    assert_eq!(result.get("X-Amz-Date").unwrap(), "20231012T144809Z");
    assert_eq!(result.get("X-Amz-User-Agent").unwrap(), "wasm");
    assert_eq!(result.get("Content-Type").unwrap(), "application/octet-stream");
    assert_eq!(result.get("Connection").unwrap(), "keep-alive");
}

#[wasm_bindgen_test]
async fn test_canonical_request() {
    let headers = generate_headers(&Date::new(&JsValue::from(1697122089958.0)));
    let url = "http://bucketname.s3.amazonaws.com/images/image.jpeg";
    let request = RequestBuilder::new(&url).headers(headers).method(Method::PUT).body("file").expect("request expected");
    let _result = _canonicalize_request(request).await.unwrap();
    //assert_eq!(result, "foo");
}
