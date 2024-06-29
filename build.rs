pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let root_dir: std::path::PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    
    let generated_dir = root_dir.join("generated");
    let js_dir = generated_dir.join("js");

    leptonic_theme::generate(generated_dir.join("leptonic"));
    println!("cargo:warning=theme written");

    std::fs::create_dir_all(js_dir.clone()).unwrap();
    println!("cargo:warning=js dir created");
    
    match std::fs::copy("./serviceWorker.js", "./generated/js/serviceWorker.js") {
        Ok(_) => println!("cargo:warning=service worker copied"),
        Err(_) => println!("cargo:error=error copying service worker"),
    }
}
