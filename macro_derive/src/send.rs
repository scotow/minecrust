extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_quote, Data, Fields, GenericParam, Generics, Index};

// Add a bound `T: Size` to every type parameter T.
pub fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(crate::types::Send));
        }
    }
    generics
}

pub fn generate_send(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Here the writer is already a &mut
                    // self.x.send(writer).await?;
                    // self.y.send(writer).await?; ...
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            self.#name.send(writer).await?;
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                }
                Fields::Unnamed(ref fields) => {
                    // Here the writer is already a &mut
                    // self.0.send(writer).await?;
                    // self.1.send(writer).await?; ...
                    let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let index = Index::from(i);
                        quote_spanned! {f.span()=>
                            self.#index.send(writer).await?;
                        }
                    });
                    quote! {
                        #(+ #recurse)*
                    }
                }
                Fields::Unit => quote!(),
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
