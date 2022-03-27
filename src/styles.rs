use dominator::class;
use once_cell::sync::Lazy;

pub static NO_USER_SELECT: Lazy<String> = Lazy::new(|| class! {
	.style(["-webkit-touch-callout", "-webkit-user-select",
		"-khtml-user-select", "-moz-user-select", "-ms-user-select",
		"user-select"], "none")
});

pub mod classname {

	pub const ALERT_STAFF: &str = "alert-staff";

	pub const BODY: &str = "body";

	pub const COLLAPSED: &str = "collapsed";

	pub const COLLAPSIBLE: &str = "collapsible";

	pub const CONTAINER: &str = "container";

	pub const DISABLED: &str = "disabled";

	pub const EXPANDED: &str = "expanded";

	pub const FRAMELESS: &str = "frameless";

	pub const ICON: &str = "icon";

	pub const TEXT_CONTENT: &str = "text";
}

pub mod component {
	pub mod button {
		use dominator::{class, pseudo};
		use lazy_static::lazy_static;

		lazy_static! {
			pub static ref ALERT_STAFF: String = class! {
				.style("--text-color", "var(--header-secondary)")
				.style("--background-color", "transparent")

				.style("align-items", "center")
				.style("background-color", "var(--background-color)")
				.style("color", "var(--text-color)")
				.style("cursor", "pointer")
				.style("display", "flex")
				.style("flex", "1 0 auto")
				.style("flex-flow", "row nowrap")
				.style("padding", "14px 13px 13px 14px")

				.pseudo!(":active", {
					.style_important("--background-color", "var(--background-tertiary)")
				})

				.pseudo!(":hover", {
					.style("--background-color", "var(--background-secondary-alt)")
					.style("--text-color", "var(--ui-text)")
				})
			};

			pub static ref BUTTON_ICON_RIGHT: String = class! { 
				.style("font-size", "20px")
				.style("margin-left", "13px")
			};

			pub static ref BUTTON_TEXT: String = class! {
				.style("font-size", "16px")
			};

			pub static ref EXPANDER_HORIZONTAL: String = class! {
				.style("--background-color", "var(--background-primary)")
				.style("--text-color", "var(--ui-inactive)")
			
				.style("background-color", "var(--background-color)")
				.style("border-bottom", "1px solid var(--background-primary)")
				.style("border-left", "1px solid var(--background-primary)")
				.style("border-right", "1px solid var(--background-primary)")
				.style("color", "var(--text-color)")
				.style("cursor", "pointer")
				.style("padding", "5px 20px 4px 20px")
				.style("text-align", "center")
			
				.pseudo!(":active", {
					.style_important("--background-color", "var(--background-tertiary)")
				})
			
				.pseudo!(":hover", {
					.style("--background-color", "var(--background-secondary-alt)")
					.style("--text-color", "var(--ui-text)")
				})
			};
		}
	}

	pub mod channel {
		use dominator::class;
		use lazy_static::lazy_static;

		lazy_static! {
			pub static ref CHANNEL: String = class! {
				.style("--avatar-large", "36pt")
				.style("--avatar-large-rounding", "5pt")
				.style("--avatar-small", "16pt")
				.style("--avatar-small-rounding", "3pt")
				.style("--message-gutter-column", "10pt")
				.style("--message-gutter-row", "5pt")
				.style("--message-padding", "10pt")

				.style("align-items", "stretch")
				.style("display", "flex")
				.style("flex", "1 0 0")
				.style("flex-flow", "column nowrap")
				.style("justify-content", "stretch")
				.style("overflow", "hidden")
			};

			pub static ref CONSOLE: String = class! {
				.style("--timestamp-span", "auto")

				.style("padding-bottom", "var(--message-padding)")
			};

			pub static ref DESCRIPTION_CONTAINER: String = class! {
				.style("background-color", "var(--background-primary)")
				.style("font-size", "16px")
				.style("max-height", "30%")
				.style("padding-bottom", "13px")
			};

			pub static ref DESCRIPTION: String = class!{
				.style("height", "100%")
				.style("padding", "0 20px")
				.style("overflow", "auto")
			};

			pub static ref MESSAGE_AREA: String = class! {
				.style("--color-date", "var(--header-secondary)")
				.style("--color-author-default", "var(--header-primary)")
				.style("--color-timestamp", "var(--ui-muted)")
				.style("--color-text-default", "var(--ui-text)")

				.style("display", "flex")
				.style("flex", "1 1 0")
				.style("flex-flow", "column nowrap")
				.style("overflow", "auto")
				.style("overflow-wrap", "break-word")
				.style("padding", "0 0 var(--message-padding) 0")
			};
		}

		pub mod action {
			use dominator::{class, pseudo};
			use lazy_static::lazy_static;

			lazy_static! {
				pub static ref ACTION: String = class! {
					.style("column-gap", "var(--message-gutter-column)")
					.style("display", "grid")
					.style("grid-template-areas", r#""avatar timestamp" "avatar content""#)
					.style("grid-template-columns", "var(--avatar-large) 1fr")
					.style("min-width", "0")
					.style("padding", "var(--message-padding)")
					.style("white-space", "pre-wrap")
				};

				pub static ref POST: String = class! {
					.style("font-style", "italic")

					.pseudo!(" em", {
						.style("font-style", "normal")
					})
				};
			}
		}

		pub mod character {
			use dominator::class;
			use lazy_static::lazy_static;
	
			lazy_static! {
				pub static ref PROFILE_PICTURE: String = class! {
					.style("border-radius", "var(--avatar-large-rounding)")
					.style("grid-area", "avatar")
					.style("height", "var(--avatar-large)")
					.style("width", "var(--avatar-large)")
				};

				pub static ref PROFILE_PICTURE_INLINE: String = class! {
					.style("border-radius", "var(--avatar-small-rounding)")
					.style("height", "var(--avatar-small)")
					.style("vertical-align", "text-top")
					.style("width", "var(--avatar-small)")
				};
			}
		}

		pub mod message {
			use dominator::{class, pseudo};
			use lazy_static::lazy_static;

			lazy_static! {
				pub static ref AD_GROUP_BACKGROUND: String = class! {
					.style("background-color", "var(--status-looking)")
					.style("height", "100%")
					.style("opacity", "0.125")
					.style("position", "absolute")
					.style("width", "100%")
				};

				pub static ref AD_GROUP_WRAPPER: String = class! {
					.style("position", "relative")
				};

				pub static ref AUTHOR: String = class! {
					.style("color", "var(--color-author-default)")
					.style("font-size", "16pt")
					.style("font-style", "normal")
					.style("grid-area", "character")
				};

				pub static ref CONTENT: String = class! {
					.style("color", "var(--color-text-default)")
					.style("flex", "1 0 0")
					.style("font-size", "16px")
					.style("min-width", "0")
					.style("white-space", "pre-wrap")
				};

				pub static ref DATE: String = class! {
					.style("color", "var(--color-date)")
					.style("font-size", "12pt")
					.style("grid-area", "timestamp")
					.style("margin-block", "0")
				};

				pub static ref DATE_INNER_TIMESTAMP: String = class! {
					.style("color", "var(--color-timestamp)")
					.style("font-size", "10pt")
				};

				pub static ref GROUP: String = class! {
					.style("padding", "var(--message-padding)")
					.style("position", "relative")
				};

				pub static ref HEAD: String = class! {
					.style("column-gap", "var(--message-gutter-column)")
					.style("display", "grid")
					.style("grid-template-areas", r#""avatar timestamp" "avatar character""#)
					.style("grid-template-columns", "var(--avatar-large) 1fr")
				};

				pub static ref MESSAGE: String = class! {
					.style("justify-items", "start")
					.style("column-gap", "var(--message-gutter-column)")
					.style("display", "flex")
					.style("flex-flow", "row nowrap")
					.style("margin-top", "var(--message-gutter-row)")
					
					.pseudo!(":nth-child(2)", {
						.style("--message-gutter-row", "2pt")
					})
				};

				// pub static ref NOTIFICATION: String = class! {
				// 	.style()
				// };

				pub static ref TIME: String = class! {
					.style("color", "var(--color-timestamp)")
					.style("font-size", "10pt")
					.style("flex", "0 0 var(--timestamp-span, var(--avatar-large))")
					.style("padding-block", "2px")
					.style("text-align", "end")
				};
			}
		}

		pub mod notification {
			use dominator::class;
			use lazy_static::lazy_static;

			lazy_static! {
				pub static ref NOTIFICATION: String = class! {
					.style("justify-items", "start")
					.style("column-gap", "var(--message-gutter-column)")
					.style("display", "flex")
					.style("flex-flow", "row nowrap")
				};

				pub static ref NOTIFICATION_AREA: String = class! {
					.style("padding", "var(--message-padding) var(--message-padding) 0 var(--message-padding)")
					.style("row-gap", "var(--message-padding)")
				};
			}
		}
	}

	pub mod input {
		use dominator::{class, pseudo};
		use lazy_static::lazy_static;

		lazy_static! {
			pub static ref TEXTAREA: String = class! {
				.style("--textarea-padding", "5px")
	
				.style("background-color", "var(--background-tertiary)")
				.style("border", "1px solid var(--background-tertiary)")
				.style("color", "var(--ui-text)")
				.style("display", "block")
				.style("font-size", "16px")
				.style("outline", "none")
				.style("padding", "calc(var(--textarea-padding) - 1px)")
				.style("resize", "none")
				.style("width", "100%")
	
				.pseudo!(":active", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
				})
	
				.pseudo!(":disabled", {
					.style_important("background-color", "var(--background-secondary-alt)")
					.style_important("border-color", "var(--background-secondary-alt)")
					.style_important("color", "var(--ui-inactive)")
					.style_important("cursor", "inherit")
				})
	
				.pseudo!(":focus", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
				})
	
				.pseudo!(":invalid", {
					.style("color", "var(--ui-danger)")
				})
	
				.pseudo!("::placeholder", {
					.style("color", "var(--ui-muted)")
				})
			};

			pub static ref TEXTBOX: String = class! {
				.style("--textbox-height", "40px")
				.style("--textbox-padding-height", "10px")
				.style("--textbox-padding-width", "5px")

				.style("background-color", "var(--background-tertiary)")
				.style("border", "1px solid var(--background-tertiary)")
				.style("color", "var(--ui-text)")
				.style("display", "block")
				.style("font-size", "16px")
				.style("outline", "none")
				.style("max-height", "var(--textbox-height)")
				.style("padding-block", "calc(var(--textbox-padding-height) - 1px)")
				.style("padding-inline", "calc(var(--textbox-padding-width) - 1px)")
				.style("width", "100%")

				.pseudo!(":active", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
				})

				.pseudo!(":disabled", {
					.style_important("background-color", "var(--background-secondary-alt)")
					.style_important("border-color", "var(--background-secondary-alt)")
					.style_important("color", "var(--ui-inactive)")
					.style_important("cursor", "inherit")
				})

				.pseudo!(":focus", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
				})

				.pseudo!(":invalid", {
					.style("color", "var(--ui-danger)")
				})

				.pseudo!("::placeholder", {
					.style("color", "var(--ui-muted)")
				})
			};

			pub static ref TEXTBOX_COMPACT: String = class! {
				.style("--textbox-padding-height", "var(--textbox-padding-width)")
			};

			pub static ref TEXTBOX_ERROR: String = class! {
				.style("border-color", "var(--ui-danger)")
				.style("color", "var(--ui-danger)")

				.pseudo!(":active", {
					.style("border-color", "var(--ui-danger)")
				})

				.pseudo!(":disabled", {
					.style_important("border-color", "var(--ui-danger)")
				})

				.pseudo!(":focus", {
					.style_important("border-color", "var(--ui-danger)")
				})
			};
		}
	}

	pub mod layout {
		use dominator::class;
		use lazy_static::lazy_static;

		lazy_static! {
			pub static ref CONTENTS: String = class! {
				.style("display", "contents")
			};

			pub static ref INLINE_BLOCK: String = class! {
				.style("display", "inline-block")
			};

			pub static ref HIDDEN: String = class! {
				.style_important("visibility", "hidden")
			};

			pub static ref VISIBLE: String = class! {
				.style_important("visibility", "visible")
			};
		}
	}

	pub mod message_box {
		use crate::styles::classname;
		use dominator::{class, pseudo};
		use lazy_static::lazy_static;

		lazy_static! {
			pub static ref FORMATTING_BAR: String = class! {
				.style("column-gap", "10px")
				.style("display", "flex")
				.style("margin-bottom", "10px")
			};
			
			pub static ref FORMATTING_BUTTON: String = class! {
				.style("--block-size", "30px")
				.style("--font-size", "16px")
				.style("--white-space", "calc(calc(var(--block-size) - var(--font-size)) / 2)")
			
				.style("color", "var(--header-secondary)")
				.style("cursor", "pointer")
				.style("font-size", "var(--font-size)")
				.style("padding", "var(--white-space)")
			
				.pseudo!(":active", {
					.style("background-color", "var(--background-secondary-alt)")
					.style("color", "var(--ui-text)")
				})
			
				.pseudo!(":focus", {
					.style("background-color", "var(--background-secondary-alt)")
					.style("color", "var(--ui-text)")
				})
			
				.pseudo!(":hover", {
					.style("background-color", "var(--background-secondary-alt)")
					.style("color", "var(--ui-text)")
				})
			
				.pseudo!(&format!(".{classname}", classname=classname::DISABLED), {
					.style_important("background-color", "initial")
					.style_important("color", "var(--ui-inactive)")
					.style_important("cursor", "inherit")
				})
			};

			pub static ref SEND_MESSAGE: String = class! {
				.style("--block-size", "52px")
				.style("--font-size", "24px")
				.style("--white-space", "calc(calc(var(--block-size) - var(--font-size)) / 2)")

				.style("align-self", "flex-end")
				.style("background-color", "var(--background-tertiary)")
				.style("color", "var(--header-secondary)")
				.style("cursor", "pointer")
				.style("font-size", "var(--font-size)")
				.style("padding", "var(--white-space)")

				.pseudo!(":active", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
					.style("color", "var(--ui-text)")
				})

				.pseudo!(":focus", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
					.style("color", "var(--ui-text)")
				})

				.pseudo!(":hover", {
					.style("background-color", "var(--background-base)")
					.style("border-color", "var(--background-base)")
					.style("color", "var(--ui-text)")
				})

				.pseudo!(&format!(".{classname}", classname=classname::DISABLED), {
					.style_important("background-color", "var(--background-tertiary)")
					.style_important("border-color", "var(--background-tertiary)")
					.style_important("color", "var(--ui-inactive)")
					.style_important("cursor", "inherit")
				})
			};
		}
	}
	
	pub mod sidebar {

	}

	pub mod view {
		use dominator::class;
		use lazy_static::lazy_static;

		lazy_static! {
			pub static ref CONTAINER: String = class! {
				.style("display", "flex")
				.style("flex", "0 1 100%")
				.style("flex-flow", "column nowrap")
				.style("height", "100%")
				.style("min-width", "0")
			};
		}
	}
}

pub mod icon {
	pub const BOLD_GLYPH: &str = "\u{E8DD}";

	pub const BULLETED_LIST_GLYPH: &str = "\u{E8FD}";

	pub const CHECK_GLYPH: &str = "\u{E73E}";

	pub const CLOSE_GLYPH: &str = "\u{E8BB}";

	pub const CONSOLE_VIEW_GLYPH: &str = "\u{E8E4}";

	pub const CONTACT_FRAME_GLYPH: &str = "\u{E8D4}";

	pub const CONTACT_SOLID_GLYPH: &str = "\u{EA8C}";

	pub const DOWN_CHEVRON_GLYPH: &str = "\u{E70D}";

	pub const EMOJI_GLYPH: &str = "\u{E76E}";

	pub const ERROR_GLYPH: &str = "\u{E783}";

	pub const EYE_GLAZE_GLYPH: &str = "\u{F19D}";

	pub const FAVOURITE_STAR_FILLED: &str = "\u{E735}";

	pub const FONT_COLOR_GLYPH: &str = "\u{E8D3}";

	pub const FONT_SIZE_DECREASE_GLYPH: &str = "\u{E8E7}";

	pub const FONT_SIZE_INCREASE_GLYPH: &str = "\u{E8E8}";

	pub const HISTORY_GLYPH: &str = "\u{E81C}";

	pub const ITALIC_GLYPH: &str = "\u{E8DB}";

	pub const KNOWLEDGE_ARTICLE_GLYPH: &str = "\u{F000}";

	pub const LEFT_CHEVRON_GLYPH: &str = "\u{E76B}";

	pub const LINK_GLYPH: &str = "\u{E71B}";

	pub const MAXIMISE_GLYPH: &str = "\u{E923}";
	
	pub const MINIMISE_GLYPH: &str = "\u{E949}";

	pub const PIN_GLYPH: &str = "\u{E718}";

	pub const PINNED_GLYPH: &str = "\u{E840}";

	pub const RIGHT_CHEVRON_GLYPH: &str = "\u{E76C}";

	pub const SEARCH_GLYPH: &str = "\u{E721}";

	pub const SEND_FILLED_GLYPH: &str = "\u{E725}";

	pub const SETTINGS_GLYPH: &str = "\u{E713}";

	pub const STATUS_CIRCLE_BLOCK_GLYPH: &str = "\u{F140}";

	pub const STRIKETHROUGH_GLYPH: &str = "\u{EDE0}";
	
	pub const UNDERLINE_GLYPH: &str = "\u{E8DC}";

	pub const UP_CHEVRON_GLYPH: &str = "\u{E70E}";
}

pub mod theme {
    use dominator::class;
    use once_cell::sync::Lazy;

	pub static CHAT_BLUE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-blue)")
	});

	pub static CHAT_BROWN: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-brown)")
	});

	pub static CHAT_CYAN: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-cyan)")
	});

	pub static CHAT_GREEN: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-green)")
	});

	pub static CHAT_ORANGE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-orange)")
	});

	pub static CHAT_PINK: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-pink)")
	});

	pub static CHAT_PURPLE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-purple)")
	});

	pub static CHAT_RED: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-red)")
	});

	pub static CHAT_YELLOW: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-yellow)")
	});

	pub static CHAT_BLACK: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-black)")
	});

	pub static CHAT_GREY: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-grey)")
	});

	pub static CHAT_WHITE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--chat-white)")
	});

	pub static SEX_CUNT_BOY: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-cunt-boy)")
	});

	pub static SEX_FEMALE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-female)")
	});

	pub static SEX_HERMAPHRODITE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-hermaphrodite)")
	});

	pub static SEX_MALE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-male)")
	});

	pub static SEX_MALE_HERM: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-male-herm)")
	});

	pub static SEX_SHEMALE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-shemale)")
	});

	pub static SEX_TRANSGENDER: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-transgender)")
	});

	pub static SEX_NONE_SET: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--sex-none-set)")
	});

	pub static STATUS_LOOKING: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-looking)")
	});

	pub static STATUS_ONLINE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-online)")
	});

	pub static STATUS_AWAY: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-away)")
	});

	pub static STATUS_IDLE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-idle)")
	});

	pub static STATUS_BUSY: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-busy)")
	});

	pub static STATUS_DND: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-dnd)")
	});

	pub static STATUS_OFFLINE: Lazy<String> = Lazy::new(|| class! {
		.style("color", "var(--status-offline)")
	});
}
