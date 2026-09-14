fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos")
        || std::env::var_os("DOCS_RS").is_some()
    {
        return;
    }

    println!("cargo:rerun-if-changed=src/bridge.m");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    let deployment_target =
        std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "12.0".to_string());

    cc::Build::new()
        .file("src/bridge.m")
        .flag("-fobjc-arc")
        .env("MACOSX_DEPLOYMENT_TARGET", deployment_target)
        .compile("bridge");

    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=MediaPlayer");
}
