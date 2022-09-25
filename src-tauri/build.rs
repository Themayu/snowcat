#[rustversion::nightly]
fn main() {
	println!("cargo:rustc-cfg=feat_array_chunks");
	tauri();
}

#[rustversion::beta]
fn main() {
	tauri();
}

#[rustversion::stable]
fn main() {
	tauri();
}

fn tauri() {
	tauri_build::build()
}
