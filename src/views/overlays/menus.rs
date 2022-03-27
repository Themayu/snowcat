use crate::App;
use discard::Discard;
use dominator::events;
use dominator::{Dom, DomHandle, EventOptions, class, clone, html, pseudo};
use futures_signals::signal::Mutable;
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::SignalVecExt;
use once_cell::sync::Lazy;
use wasm_bindgen_futures::spawn_local;
use std::cell::{RefCell, Cell};
use std::fmt;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use web_sys::{HtmlElement, Element};

static CONTEXT_MENU: Lazy<String> = Lazy::new(|| class! {
	.style("background-color", "var(--background-floating)")
	.style("position", "absolute")
	.style("overflow-y", "auto")
	.style("max-height", "600px")
	.style("visibility", "hidden")

	.pseudo!(" > *", {
		.style("opacity", "0")
		.style("transition", "opacity 400ms cubic-bezier(0.660, 0.000, 0.440, 1.000) 500ms")
	})
});

static CONTEXT_MENU_CONTAINER: Lazy<String> = Lazy::new(|| class! {
	.style("--padding", "10px")

	.style("bottom", "var(--padding)")
	.style("left", "var(--padding)")
	.style("position", "absolute")
	.style("overflow", "hidden")
	.style("right", "var(--padding)")
	.style("top", "var(--padding)")
});

static CONTEXT_MENU_VISIBLE: Lazy<String> = Lazy::new(|| class! {
	.style("height", "var(--height, 30px)")
	.style("left", "var(--x, 0px)")
	.style("top", "var(--y, 0px)")
	.style("transition-delay", "125ms, 0ms, 125ms, 0ms")
	.style("transition-duration", "500ms, 500ms, 500ms, 500ms")
	.style("transition-property", "height, left, top, width")
	.style("transition-timing-function", "cubic-bezier(0.660, 0.000, 0.440, 1.000)")
	.style("visibility", "visible")
	.style("width", "var(--width, 30px)")

	.pseudo!(" > *", {
		.style("opacity", "1")
	})
});

pub struct BoundingBox {
	// position of top-left corner
	pub x: i32,
	pub y: i32,

	// edge positions
	pub bottom: i32,
	pub left: i32,
	pub right: i32,
	pub top: i32,

	// size
	pub height: i32,
	pub width: i32,
}

#[derive(Debug, Clone)]
pub struct ContextMenu {
	container: Rc<ContextMenuContainer>,
	id: usize,
}

impl ContextMenu {
	pub fn close(&self) {
		self.container.remove(&self.id);
	}
}

pub struct ContextMenuContainer {
	counter: AtomicUsize,
	element: RwLock<Option<HtmlElement>>,
	items: MutableBTreeMap<usize, Rc<dyn Fn(ContextMenu) -> Dom>>,

	item_count: Mutable<usize>,
}

impl ContextMenuContainer {
	pub fn new() -> Rc<Self> {
		Rc::new(ContextMenuContainer {
			counter: AtomicUsize::new(0),
			element: RwLock::new(None),
			items: MutableBTreeMap::new(),
			
			item_count: Mutable::new(0),
		})
	}

	pub fn add<Builder>(self: &Rc<Self>, builder: Builder) -> usize
	where Builder: 'static + Fn(ContextMenu) -> Dom, {
		let id = self.counter.fetch_add(1, Ordering::SeqCst);
		let mut item_count = self.item_count.lock_mut();

		self.items.lock_mut().insert_cloned(id, Rc::new(builder));
		*item_count = item_count.saturating_add(1);

		id
	}

	pub fn as_element(self: &Rc<Self>) -> HtmlElement {
		self.element.read().expect("a healthy element lock").as_ref().expect("the container to have been inserted into DOM").clone()
	}

	pub fn clear(self: &Rc<Self>) {
		self.items.lock_mut().clear();
		self.item_count.set(0);
	}

	pub fn render(self: &Rc<Self>) -> Dom {
		let container = self.clone();

		html!("div", {
			.class(&*self::CONTEXT_MENU_CONTAINER)
			.children_signal_vec(self.items.entries_cloned().map(clone!(container => move |(id, builder)| {		
				let context_menu = ContextMenu {
					container: container.clone(),
					id,
				};

				builder(context_menu)
			})))

			.event_with_options(&EventOptions::bubbles(), clone!(container => move |_: events::Click| {
				container.clear();
			}))

			.event_with_options(&EventOptions::bubbles(), clone!(container => move |_: events::ContextMenu| {
				container.clear();
			}))

			.style_signal("display", self.item_count.signal_ref(|count| Some(if count > &0 { "block" } else { "none" })))
			.after_inserted(move |element| {
				*container.element.write().expect("a healthy element lock") = Some(element);
			})
		})
	}

	pub fn remove(self: &Rc<Self>, handle: &usize) {
		self.items.lock_mut().remove(handle);
		
		let mut item_count = self.item_count.lock_mut();
		*item_count = item_count.saturating_sub(1);
	}
}

impl fmt::Debug for ContextMenuContainer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let items = self.items.lock_ref().len();

		f.debug_struct("ContextMenuContainer")
			.field("counter", &self.counter)
			.field("items", &format!("MutableBTreeMap ({} {})", items, if items == 1 { "item" } else { "items" }))
			.finish()
	}
}

/// Where to place the context menu in relation to the target object.
/// 
/// Object placement rules are as follows:
/// - `BesideTarget` and `AtCursor` will attempt to place the context menu on the bottom right of the final position.
///   - If the available horizontal space is insufficient, the context menu will be moved to the left of the final
///     position.
///   - If the available vertical space is insufficient, the context menu will be moved to above the final position.
///     - If the available space is still insufficient, the context menu will be squashed down so as to not overflow
///       the overlay host.
pub enum ContextMenuPosition {
	/// Place the context menu above the target, indicating it with an arrow.
	AboveTarget(Element),

	/// Place the context menu below the target, indicating it with an arrow.
	BelowTarget(Element),

	/// Place the context menu either side of the target, indicating it with an arrow.
	BesideTarget(Element),

	/// Place the context menu at the current position of the mouse cursor.
	AtCursor { x: i32, y: i32 },
}

pub struct ContextMenuState {
	handle: Rc<RefCell<Option<DomHandle>>>,

	x: Mutable<Option<i32>>,
	y: Mutable<Option<i32>>,
	width: Mutable<Option<i32>>,
	height: Mutable<Option<i32>>,

	visible: Mutable<bool>,
}

impl ContextMenuState {
	pub fn close(&self) {
		self.handle.borrow_mut().take().unwrap().discard();
	}
}

/// Build a context menu at the given location, by 
pub fn context_menu<ContentBuilder>(
	app: Arc<App>,
	position: ContextMenuPosition,
	content_builder: ContentBuilder,
)
where ContentBuilder: 'static + FnOnce(Rc<ContextMenuState>) -> Dom, {
	let content_builder = Rc::new(Cell::new(Some(content_builder)));
	let overlay = &app.overlays.context_menu;
	let position = Rc::new(position);

	let bounds = Rc::new(measure(&app.overlays.context_menu.as_element()));
	let state = Rc::new(ContextMenuState {
		handle: Rc::new(RefCell::new(None)),

		x: Mutable::new(None),
		y: Mutable::new(None),
		width: Mutable::new(None),
		height: Mutable::new(None),

		visible: Mutable::new(false),
	});

	overlay.add(move |menu| html!("div", {
		.after_inserted(clone!(bounds, position, state => move |element| {
			let (menu_width, menu_height) = measure_dimensions(&element);
			let (overlay_x, overlay_y, menu_height) = match position.as_ref() {
				ContextMenuPosition::AboveTarget(target) => {
					(0, 0, 600)
				},
		
				ContextMenuPosition::BelowTarget(target) => {
					(0, 0, 600)
				},
		
				ContextMenuPosition::BesideTarget(target) => {
					(0, 0, 600)
				}
		
				ContextMenuPosition::AtCursor { x, y } => {
					let too_long = x.saturating_add(menu_width) > bounds.right;
					let too_tall = y.saturating_add(menu_height) > bounds.bottom;

					let (menu_height, y) = match too_tall {
						true => {
							let overflow = (bounds.top - menu_height).max(0).saturating_abs();
							let height = menu_height - overflow;

							(height, y.sub(height))
						},

						false => (menu_height, *y),
					};

					let x = match too_long {
						true => x.sub(menu_width).min(10),
						false => *x,
					};

					(x, y, menu_height)
				},
			};

			state.visible.set(true);

			spawn_local(async move {
				let state = state.clone();

				state.x.set(Some(overlay_x));
				state.y.set(Some(overlay_y));
				state.width.set(Some(menu_width));
				state.height.set(Some(menu_height));
			});

			// log::debug!("build context menu at ({x}, {y})", x = content_x, y = content_y);
		}))

		.class(&*self::CONTEXT_MENU)
		.class_signal(&*self::CONTEXT_MENU_VISIBLE, clone!(state => state.visible.signal()))
		.child(clone!(state, content_builder => (content_builder.take().unwrap())(state)))

		.style_signal("--x", clone!(state => {
			state.x.signal_ref(|x| x.map(|value| format!("{}px", value)))
		}))

		.style_signal("--y", clone!(state => {
			state.y.signal_ref(|y| y.map(|value| format!("{}px", value)))
		}))

		.style_signal("--width", clone!(state => {
			state.width.signal_ref(|width| width.map(|value| format!("{}px", value)))
		}))

		.style_signal("--height", clone!(state => {
			state.height.signal_ref(|height| height.map(|value| format!("{}px", value)))
		}))
	}));
}

fn measure(element: &Element) -> BoundingBox {
	let bounding_box = element.get_bounding_client_rect();

	BoundingBox {
		x: bounding_box.x() as i32,
		y: bounding_box.y() as i32,

		bottom: bounding_box.bottom() as i32,
		left: bounding_box.left() as i32,
		right: bounding_box.right() as i32,
		top: bounding_box.top() as i32,

		height: bounding_box.height() as i32,
		width: bounding_box.width() as i32,
	}
}

fn measure_dimensions(element: &HtmlElement) -> (i32, i32) {
	let bounding_box = element.get_bounding_client_rect();
	
	let height = bounding_box.height() as i32;
	let width = bounding_box.width() as i32;

	(width.max(300), height.max(600))
}
