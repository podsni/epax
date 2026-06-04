fn download_file(url: &str, dest: &std::path::Path) {
    let status = std::process::Command::new("curl")
        .args(["-L", "-o", dest.to_str().unwrap(), url])
        .status()
        .expect("failed to execute curl");
    if !status.success() {
        panic!("failed to download model from {}", url);
    }
}

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

    if target.contains("windows") && target.contains("msvc") {
        println!("cargo:rustc-link-arg=/nodefaultlib:libcmt");
        println!("cargo:rustc-link-arg=/nodefaultlib:libcpmt");
    }

    let parse = std::env::var("CARGO_FEATURE_PARSE").is_ok();
    if parse {
        // Ensure models directory exists
        let out_dir = std::path::Path::new("models");
        std::fs::create_dir_all(out_dir).unwrap();

        let det_path = out_dir.join("text-detection.rten");
        let rec_path = out_dir.join("text-recognition.rten");
        let pad_det_path = out_dir.join("PP-OCRv5_mobile_det_fp16.mnn");
        let pad_rec_path = out_dir.join("PP-OCRv5_mobile_rec_fp16.mnn");
        let pad_keys_path = out_dir.join("ppocr_keys_v5.txt");

        if !det_path.exists() {
            println!("cargo:warning=Downloading text-detection.rten...");
            download_file("https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten", &det_path);
        }
        if !rec_path.exists() {
            println!("cargo:warning=Downloading text-recognition.rten...");
            download_file("https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten", &rec_path);
        }
        if !pad_det_path.exists() {
            println!("cargo:warning=Downloading PP-OCRv5_mobile_det_fp16.mnn...");
            download_file("https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_det_fp16.mnn", &pad_det_path);
        }
        if !pad_rec_path.exists() {
            println!("cargo:warning=Downloading PP-OCRv5_mobile_rec_fp16.mnn...");
            download_file("https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/PP-OCRv5_mobile_rec_fp16.mnn", &pad_rec_path);
        }
        if !pad_keys_path.exists() {
            println!("cargo:warning=Downloading ppocr_keys_v5.txt...");
            download_file("https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/next/models/ppocr_keys_v5.txt", &pad_keys_path);
        }
    }
}
