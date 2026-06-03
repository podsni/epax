fn main() {
    // The vendored `unrar` C++ sources call Win32 registry, token, security and
    // crypto APIs (RegOpenKeyExW, OpenProcessToken, CryptGenRandom, …) that live
    // in advapi32.lib, but `unrar-sys` does not request that library on MSVC, so
    // linking fails with unresolved externals. Add it ourselves when the RAR
    // feature is built for a Windows MSVC target.
    let target = std::env::var("TARGET").unwrap_or_default();
    let rar = std::env::var("CARGO_FEATURE_RAR").is_ok();
    if rar && target.contains("windows") && target.contains("msvc") {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
}
