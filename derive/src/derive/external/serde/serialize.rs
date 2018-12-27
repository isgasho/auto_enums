use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::*;

pub(crate) const NAME: &[&str] = &["serde::Serialize"];

pub(crate) fn derive(data: &Data) -> Result<TokenStream> {
    let root = std_root();
    let ser = quote!(::serde::ser);

    derive_trait!(
        data,
        parse_quote!(#ser::Serialize)?,
        parse_quote! {
            trait Serialize {
                #[inline]
                fn serialize<__S>(&self, serializer: __S) -> #root::result::Result<__S::Ok, __S::Error>
                where
                    __S: #ser::Serializer;
            }
        }?,
    )
}
