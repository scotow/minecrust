extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_quote, Data, Fields, GenericParam, Generics, Index};

// Add a bound `T: Size` to every type parameter T.
pub fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(crate::types::Size));
        }
    }
    generics
}

pub fn size_sum(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // 0 + self.x.size() + self.y.size() + self.z.size()
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            self.#name.size()
                        }
                    });
                    quote! {
                        crate::types::VarInt::new(0) #(+ #recurse)*
                    }
                }
                Fields::Unnamed(ref fields) => {
                    // 0 + self.0.size() + self.1.size() + self.2.size()
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = Index::from(i);
                        quote_spanned! {f.span()=>
                            self.#index.size()
                        }
                    });
                    quote! {
                        crate::types::VarInt::new(0) #(+ #recurse)*
                    }
                }
                Fields::Unit => quote!(crate::types::VarInt::new(0)),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
