fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-weak_framework");
    println!("cargo:rustc-link-arg=GameController");
    println!("cargo:rustc-link-arg=-weak_framework");
    println!("cargo:rustc-link-arg=Metal");
    println!("cargo:rustc-link-arg=-weak_framework");
    println!("cargo:rustc-link-arg=QuartzCore");
    println!("cargo:rustc-link-arg=-weak_framework");
    println!("cargo:rustc-link-arg=UniformTypeIdentifiers");
}
