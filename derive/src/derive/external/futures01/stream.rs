use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::*;

pub(crate) const NAME: &[&str] = &["futures01::Stream"];

pub(crate) fn enum_derive(data: &syn::ItemEnum) -> Result<TokenStream> {
    EnumData::parse(data, false, true).map(|data| stream(&data, &std_root()))
}

fn stream(data: &EnumData<'_>, root: &TokenStream) -> TokenStream {
    let EnumData {
        name,
        impl_generics,
        ty_generics,
        where_clause,
        variants,
        fields,
    } = data;

    let crate_ = quote!(::futures);
    let trait_ = quote!(#crate_::stream::Stream);
    let fst = &fields[0];

    let where_clause = fields
        .iter()
        .skip(1)
        .fold(quote!(#where_clause #fst: #trait_,), |t, f| {
            t.extend_and_return(quote!(#f: #trait_<Item = <#fst as #trait_>::Item, Error = <#fst as #trait_>::Error>,))
        });

    quote! {
        impl #impl_generics #trait_ for #name #ty_generics #where_clause {
            type Item = <#fst as #trait_>::Item;
            type Error = <#fst as #trait_>::Error;
            #[inline]
            fn poll(&mut self) -> #crate_::Poll<#root::option::Option<Self::Item>, Self::Error> {
                match self { #(#variants(x) => x.poll(),)* }
            }
        }
    }
}