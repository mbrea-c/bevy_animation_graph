use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parenthesized, parse_macro_input, Attribute, Data, DeriveInput, Fields, Meta, Path, Variant,
};

#[proc_macro_derive(ValueWrapper, attributes(unwrap_error, trivial_copy))]
pub fn into_variants(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (error_type, error_variant) = parse_error_attribute(&input.attrs);

    let Data::Enum(data_enum) = &input.data else {
        panic!("IntoVariants only works on enums.");
    };

    let (into_fns, as_fns, from_impls) = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let fields = match &variant.fields {
                Fields::Unnamed(fields_unnamed) => fields_unnamed,
                _ => panic!("Macro only supports unnamed fields."),
            };
            if fields.unnamed.len() != 1 {
                panic!("Macro only supports enums with one field.");
            }

            let field = fields.unnamed.first().unwrap();
            let return_type = &field.ty;
            let field_pattern = quote! { (x) };

            let into_fn_name = syn::Ident::new(
                &format!("into_{}", to_snake_case(&variant_name.to_string())),
                variant_name.span(),
            );

            let as_fn_name = syn::Ident::new(
                &format!("as_{}", to_snake_case(&variant_name.to_string())),
                variant_name.span(),
            );

            let into_fn = quote! {
                #[must_use]
                pub fn #into_fn_name(self) -> Result<#return_type, #error_type> {
                    match self {
                        Self::#variant_name #field_pattern => Ok(x),
                        _ => Err(#error_type::#error_variant(stringify!(#variant_name).to_string(), std::any::type_name::<Self>().to_string())),
                    }
                }
            };

            let as_fn = if is_trivial_copy(variant) {
                quote! {
                    #[must_use]
                    pub fn #as_fn_name(&self) -> Result<#return_type, #error_type> {
                        match self {
                            Self::#variant_name #field_pattern => Ok(x.clone()),
                            _ => Err(#error_type::#error_variant(stringify!(#variant_name).to_string(), std::any::type_name::<Self>().to_string())),
                        }
                    }
                }
            } else {
                quote! {
                    #[must_use]
                    pub fn #as_fn_name(&self) -> Result<&#return_type, #error_type> {
                        match self {
                            Self::#variant_name #field_pattern => Ok(x),
                            _ => Err(#error_type::#error_variant(stringify!(#variant_name).to_string(), std::any::type_name::<Self>().to_string())),
                        }
                    }
                }
            };

            let from_impl = quote! {
                impl From<#return_type> for DataValue {
                    fn from(value: #return_type) -> Self {
                        Self::#variant_name(value)
                    }
                }
            };

            (into_fn, as_fn, from_impl)
        })
        .fold(
            (vec![], vec![], vec![]),
            |(mut into_fns, mut as_fns, mut from_impls), (into_fn, as_fn, from_impl)| {
                into_fns.push(into_fn);
                as_fns.push(as_fn);
                from_impls.push(from_impl);

                (into_fns, as_fns, from_impls)
            },
        );

    TokenStream::from(quote! {
        impl #name {
            #(#into_fns)*

            #(#as_fns)*
        }

        #(#from_impls)*
    })
}

fn is_trivial_copy(variant: &Variant) -> bool {
    variant.attrs.iter().any(|attr| {
        if !attr.path().is_ident("trivial_copy") {
            return false;
        }

        match &attr.meta {
            Meta::Path(_) => {}
            _ => panic!("trivial_copy cannot contain arguments."),
        }

        true
    })
}

fn parse_error_attribute(attrs: &[Attribute]) -> (Path, syn::Ident) {
    let Some((Some(error_type), Some(error_variant))) = attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::List(meta_list) => Some(meta_list),
            _ => None,
        })
        .filter(|meta_list| meta_list.path.is_ident("unwrap_error"))
        .map(|meta_list| {
            let mut error_type = None;
            let mut error_variant = None;

            let _ = meta_list.parse_nested_meta(|meta| {
                if meta.path.is_ident("error") {
                    let content;
                    parenthesized!(content in meta.input);

                    let path: Path = content.parse()?;
                    error_type = Some(path);
                }

                if meta.path.is_ident("variant") {
                    let content;
                    parenthesized!(content in meta.input);

                    let ident: syn::Ident = content.parse()?;
                    error_variant = Some(ident);
                }

                Ok(())
            });

            (error_type, error_variant)
        })
        .next()
    else {
        panic!("No error or variant found!")
    };

    (error_type, error_variant)
}

fn to_snake_case(input: &str) -> String {
    let mut s = String::new();
    let mut prev_is_upper = false;

    for (i, c) in input.char_indices() {
        if c.is_uppercase() {
            if i != 0 && !prev_is_upper {
                s.push('_');
            }
            s.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            s.push(c);
            prev_is_upper = false;
        }
    }

    s
}
