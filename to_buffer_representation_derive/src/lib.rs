extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

// create a derive macro for ToBufferRepresentation
#[proc_macro_derive(ToBufferRepresentation)]
pub fn derive_to_buffer_representation(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_to_buffer_representation(&ast)
}

fn impl_to_buffer_representation(ast: &DeriveInput) -> TokenStream {
    // get the name of the struct that is being derived without the added reference
    let name = &ast.ident;
    let gen = quote! {
        impl ToBufferRepresentation for #name {
            fn to_bits(&self) -> &[u8] {
                bytemuck::cast_slice(array::from_ref(self))
            }
        }
    };
    gen.into()
}
