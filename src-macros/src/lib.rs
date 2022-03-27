mod discriminate;

use proc_macro::TokenStream;

/// Create a "discriminant" enum for the current enum: a data-less version of
/// the enum that can be used to match solely on the variant names.
/// 
/// # Example
/// ```rust
/// use snowcat_macros::discriminate;
/// 
/// #[discriminate]
/// enum MessageType {
///     Action(String),
///     Ad(String),
///     Message(String),
/// }
/// 
/// let message = MessageType::Ad(String::from("ad name here"));
/// 
/// assert_eq!(message.discriminant(), MessageTypeDiscriminant::Ad);
/// assert_ne!(message.discriminant(), MessageTypeDiscriminant::Message);
/// ```
#[proc_macro_attribute]
pub fn discriminate(_: TokenStream, input: TokenStream) -> TokenStream {
	discriminate::discriminate(input)
}
