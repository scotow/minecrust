extern crate proc_macro;

use quote::quote;

use syn::{parse_macro_input, DeriveInput};

mod send;
mod size;

#[proc_macro_derive(Send)]
pub fn derive_send(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let generics = send::add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to send each field
    let send = send::generate_send(&input.data);

    let expanded = quote! {
        // The generated impl.
        #[async_trait::async_trait]
        impl #impl_generics crate::types::Send for #struct_name #ty_generics #where_clause {
            async fn send<W: futures::io::AsyncWrite + std::marker::Send + std::marker::Unpin>(&self, writer: &mut W) -> anyhow::Result<()> {
                use crate::types::Send;
                #send
                Ok(())
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Size)]
pub fn derive_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let generics = size::add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to sum up the size of each field.
    let sum = size::size_sum(&input.data);

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
