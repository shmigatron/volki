fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=/opt/homebrew/opt/openssl/lib");
        println!("cargo:rustc-link-search=/usr/local/opt/openssl/lib");
    }
}
