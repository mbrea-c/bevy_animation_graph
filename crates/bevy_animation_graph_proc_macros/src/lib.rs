mod uuid_wrapper;
mod value_wrapper;

use proc_macro::TokenStream;

use crate::{uuid_wrapper::uuid_wrapper, value_wrapper::value_wrapper};

#[proc_macro_derive(UuidWrapper, attributes(uuid))]
pub fn derive_uuid_wrapper(input: TokenStream) -> TokenStream {
    uuid_wrapper(input)
}

#[proc_macro_derive(ValueWrapper, attributes(unwrap_error, trivial_copy))]
pub fn derive_value_wrapper(input: TokenStream) -> TokenStream {
    value_wrapper(input)
}
