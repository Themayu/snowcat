#[rustversion::nightly]
fn main() {
	println!("cargo:rustc-cfg=feat_future_join");
}

#[rustversion::beta]
fn main() {}

#[rustversion::stable]
fn main() {}
