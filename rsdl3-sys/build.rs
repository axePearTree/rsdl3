fn main() {
    println!("cargo:rustc-link-lib=SDL3");

    #[cfg(feature = "image")]
    println!("cargo:rustc-link-lib=SDL3_image");
}
