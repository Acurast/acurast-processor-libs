fn main() {
    println!("cargo:rustc-link-lib=framework=Security");
    uniffi::generate_scaffolding("./src/acurastsdk.udl").unwrap();
}