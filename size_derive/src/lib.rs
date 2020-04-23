extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index};

#[proc_macro_derive(Size)]
pub fn derive_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to sum up the size of each field.
    let sum = size_sum(&input.data);

    let expanded = quote! {
        // The generated impl.
        impl #impl_generics crate::types::Size for #struct_name #ty_generics #where_clause {
            fn size(&self) -> crate::types::VarInt {
                use crate::types::Size;
                #sum
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

// Add a bound `T: Size` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(crate::types::Size));
        }
    }
    generics
}

fn size_sum(data: &Data) -> TokenStream {
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
                Fields::Unit => {
                    quote!(crate::types::VarInt::new(0))
                }
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}