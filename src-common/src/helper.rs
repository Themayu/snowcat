use serde::Deserialize;

/// A helper class to assist with deserialising and serializing the built-in
/// `Result` type as an untagged enum.
/// 
/// To deserialize instances of `Result` as untagged:
/// ```rust,ignored
/// use crate::helpers::ResultImpl;
/// use serde::Deserialize;
/// use serde_json::{Deserializer, json};
/// 
/// #[derive(Debug, Deserialize)]
/// struct Body {
/// 	value: String,
/// }
/// 
/// struct Error {
/// 	error: String,
/// }
/// 
/// let body_str = r#"{"value": "Hello, World!"}"#;
/// let error_str = r#"{"error": "An error occurred."}"#;
/// 
/// let mut body_deserializer = Deserializer::from_str(&body_str);
/// let deserialized_body = ResultImpl::deserialize(&mut body_deserializer);
/// 
/// let mut error_deserializer = Deserializer::from_str(&error_str);
/// let deserialized_error = ResultImpl::deserialize(&mut error_deserializer);
/// 
/// assert_eq!(deserialized_body, Ok(Body { value: "Hello, World!" }));
/// assert_eq!(deserialized_error, Err(Error { error: "An error occurred." }));
/// ```
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(remote = "Result")]
#[serde(untagged)]
pub enum ResultImpl<T, E> {
	Ok(T),
	Err(E),
}
