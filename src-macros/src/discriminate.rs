use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, ItemEnum, Ident, Variant, Fields, Pat, PatWild, Token};

pub fn discriminate(input: TokenStream) -> TokenStream {
	let input: ItemEnum = parse_macro_input!(input as ItemEnum);
	let visibility = &input.vis;

	let type_ident = &input.ident;
	let discriminant_ident = {
		Ident::new(&format!("{type_ident}Discriminant"), Span::call_site())
	};

	let arms = {
		let mut arms = vec![];

		input.variants.iter().for_each(|variant| {
			let name = &variant.ident;
			
			let pattern = match &variant.fields {
				Fields::Unit => quote! {
					#type_ident::#name => #discriminant_ident::#name
				},

				Fields::Named(_) => quote! {
					#type_ident::#name { .. } => #discriminant_ident::#name
				},

				Fields::Unnamed(fields) => {
					let mut elements = Punctuated::<Pat, Comma>::new();

					fields.unnamed.pairs().for_each(|_| {
						elements.push(Pat::Wild(PatWild {
							attrs: vec![],
							underscore_token: Token![_]([Span::call_site()]),
						}))
					});

					quote! {
						#type_ident::#name (#elements) => #discriminant_ident::#name
					}
				}
			};

			arms.push(pattern);
		});

		arms
	};

	let variants = {
		let mut variants = Punctuated::<Variant, Comma>::new();
		input.variants.iter().for_each(|variant| {
			variants.push(Variant {
				attrs: vec![],
				ident: variant.ident.clone(),
				fields: Fields::Unit,
				discriminant: variant.discriminant.clone(),
			})
		});

		variants
	};

	let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
	let to_discriminant_impl = quote! {
		impl #impl_generics #type_ident #type_generics #where_clause {
			#visibility fn discriminant(&self) -> #discriminant_ident {
				match self {
					#(#arms),*
				}
			}
		}
	};

	let discriminant_display_impl = {
		let arms = variants.iter().map(|variant| {
			let arm_ident = &variant.ident;
			
			quote! {
				#discriminant_ident::#arm_ident => write!(f, "{}", stringify!(#arm_ident))
			}
		}).collect::<Vec<_>>();

		quote! {
			match self {
				#(#arms),*
			}
		}
	};

	let discriminant_enum = quote! {
		#[derive(Debug, Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
		#visibility enum #discriminant_ident {
			#variants
		}

		impl ::std::fmt::Display for #discriminant_ident {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				#discriminant_display_impl
			}
		}

		impl ::std::convert::From<#type_ident> for #discriminant_ident {
			fn from(source: #type_ident) -> Self {
				source.discriminant()
			}
		}
	};

	let assembled = quote! {
		#input

		#to_discriminant_impl

		#discriminant_enum
	};

	assembled.into()
}
