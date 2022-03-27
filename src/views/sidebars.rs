mod navigation;

pub use navigation::{NavigationView, render as render_navigation};
use dominator::class;
use once_cell::sync::Lazy;

pub static SIDEBAR_CONTAINER: Lazy<String> = Lazy::new(|| class! {
	.style("--width", "250px")

	.style("background-color", "var(--background-secondary-alt)")
	.style("flex", "0 0 auto")
	.style("height", "100%")
	.style("overflow", "hidden")
	.style("width", "var(--width)")
});

pub static SIDEBAR: Lazy<String> = Lazy::new(|| class! {
	.style("display", "flex")
	.style("flex-flow", "column nowrap")
	.style("height", "100%")
	.style("width", "var(--width)")
});
