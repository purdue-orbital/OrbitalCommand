
fn main() {
    println!("cargo:rustc-link-lib=dylib=bladeRF");
    // TODO make this build on other peoples computers
    println!("cargo:rustc-link-search=native=/home/nicholasball/Documents/GitHub/OrbitalCommand/radio/lib");
}