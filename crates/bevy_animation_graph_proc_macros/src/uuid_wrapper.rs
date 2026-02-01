use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Index, parse_macro_input};

pub(crate) fn uuid_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data_struct) = &input.data else {
        return error(&input, "UuidWrapper only works on structs.");
    };

    let Some((uuid_field_idx, uuid_field)) = data_struct
        .fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.attrs.iter().any(|attr| attr.path().is_ident("uuid")))
    else {
        return error(
            &input,
            "UuidWrapper requires an #[uuid] annotation on one of the fields",
        );
    };

    let (uuid_field_accessor, constructor) = if let Some(ident) = uuid_field.ident.as_ref() {
        let accessor = quote! {#ident};
        let constructor = quote! { #name { #ident: uuid } };
        (accessor, constructor)
    } else {
        let uuid_field_index = Index::from(uuid_field_idx);
        let accessor = quote! {#uuid_field_index};
        let constructor = quote! { #name(uuid) };
        (accessor, constructor)
    };

    TokenStream::from(quote! {
        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                self.#uuid_field_accessor.serialize(serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let uuid = ::uuid::Uuid::deserialize(deserializer)?;
                Ok(#constructor)
            }
        }

        impl #name {
            pub fn uuid(&self) -> ::uuid::Uuid {
                self.#uuid_field_accessor
            }
        }

        impl From<::uuid::Uuid> for #name {
            fn from(value: ::uuid::Uuid) -> Self {
                Self(value)
            }
        }
    })
}

fn error(input: &DeriveInput, msg: impl std::fmt::Display) -> TokenStream {
    syn::Error::new_spanned(input, msg)
        .to_compile_error()
        .into()
}
