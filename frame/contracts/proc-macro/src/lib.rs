// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Proc macros used in the contracts module.

#![no_std]

extern crate alloc;

use proc_macro2::{TokenStream, Span};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Ident, ItemStruct};
use alloc::string::ToString;
use alloc::vec::Vec;
use heck::SnakeCase;

/// This derives `Debug` for a struct where each field must be of some numeric type.
/// It interprets each field as its represents some weight and formats it as times so that
/// it is readable by humans.
#[proc_macro_derive(WeightDebug)]
pub fn derive_weight_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	derive_debug(input, format_weight)
}

/// This is basically identical to the std libs Debug derive but without adding any
/// bounds to existing generics.
#[proc_macro_derive(ScheduleDebug)]
pub fn derive_schedule_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	derive_debug(input, format_default)
}

fn derive_debug(
	input: proc_macro::TokenStream,
	fmt: impl Fn(&Ident) -> TokenStream
) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
	let data = if let Data::Struct(data) = &input.data {
		data
	} else {
		return quote_spanned! {
			name.span() =>
			compile_error!("WeightDebug is only supported for structs.");
		}.into();
	};

	#[cfg(feature = "full")]
	let fields = iterate_fields(data, fmt);

	#[cfg(not(feature = "full"))]
	let fields = {
		drop(fmt);
		drop(data);
		TokenStream::new()
	};

	let tokens = quote! {
		impl #impl_generics core::fmt::Debug for #name #ty_generics #where_clause {
			fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
				use ::sp_runtime::{FixedPointNumber, FixedU128 as Fixed};
				let mut formatter = formatter.debug_struct(stringify!(#name));
				#fields
				formatter.finish()
			}
		}
	};

	tokens.into()
}

/// This is only used then the `full` feature is activated.
#[cfg(feature = "full")]
fn iterate_fields(data: &DataStruct, fmt: impl Fn(&Ident) -> TokenStream) -> TokenStream {
	match &data.fields {
		Fields::Named(fields) => {
			let recurse = fields.named
			.iter()
			.filter_map(|f| {
				let name = f.ident.as_ref()?;
				if name.to_string().starts_with('_') {
					return None;
				}
				let value = fmt(name);
				let ret = quote_spanned!{ f.span() =>
					formatter.field(stringify!(#name), #value);
				};
				Some(ret)
			});
			quote!{
				#( #recurse )*
			}
		}
		Fields::Unnamed(fields) => quote_spanned!{
			fields.span() =>
			compile_error!("Unnamed fields are not supported")
		},
		Fields::Unit => quote!(),
	}
}

fn format_weight(field: &Ident) -> TokenStream {
	quote_spanned! { field.span() =>
		&if self.#field > 1_000_000_000 {
			format!(
				"{:.1?} ms",
				Fixed::saturating_from_rational(self.#field, 1_000_000_000).to_fraction()
			)
		} else if self.#field > 1_000_000 {
			format!(
				"{:.1?} µs",
				Fixed::saturating_from_rational(self.#field, 1_000_000).to_fraction()
			)
		} else if self.#field > 1_000 {
			format!(
				"{:.1?} ns",
				Fixed::saturating_from_rational(self.#field, 1_000).to_fraction()
			)
		} else {
			format!("{} ps", self.#field)
		}
	}
}

fn format_default(field: &Ident) -> TokenStream {
	quote_spanned! { field.span() =>
		&self.#field
	}
}

#[proc_macro_derive(HostDebug)]
pub fn host_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let item_struct = parse_macro_input!(input as ItemStruct);
	let ident = &item_struct.ident;
	let ident_name = ident.to_string().to_snake_case();
	let fields = &item_struct.fields;

	let output = if let Fields::Named(ref fields_name) = fields {
		let get_self: Vec<_> = fields_name.named.iter().map(|field| {
			let field_name = field.ident.as_ref().unwrap();
			quote! {
                &self.#field_name
            }
		}).collect();

		quote! {
            impl fmt::Debug for #ident {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    f.write_fmt(format_args!("{}({:?})", #ident_name,(#(#get_self),*)))
                }
            }
        }
	} else {
		panic!("wrong struct type!");
	};

	output.into()
}

#[proc_macro_derive(Wrap)]
pub fn wrap(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let item_struct = parse_macro_input!(input as ItemStruct);
	let ident = &item_struct.ident;
	let wrapped_ident = Ident::new(&ident.to_string(), Span::call_site());
	let output = quote! {
		impl Wrapper for #ident {
			fn wrap(&self) -> EnvTrace {
        		EnvTrace::#wrapped_ident(self.clone())
    		}
		}
	};
	output.into()
}
