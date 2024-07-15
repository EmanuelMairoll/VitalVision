use uniffi_bindgen::bindings::TargetLanguage::{Kotlin, Swift, Python};
use uniffi_bindgen::generate_bindings;

fn main() {
    let udl_file = "./src/vvcore.udl";
    let out_dir = "./bindings/";
    uniffi_build::generate_scaffolding(udl_file).unwrap();
    generate_bindings(
        udl_file.into(),
        None,
        vec![Swift, Kotlin, Python],
        Some(out_dir.into()),
        None,
        None,
        true,
    )
    .unwrap();
}
