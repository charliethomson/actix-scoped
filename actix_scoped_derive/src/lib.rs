mod util;
mod parse;
mod gen;

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};

use syn::{parse_macro_input, DeriveInput};
use quote::quote;
use crate::gen::{create_from_request, create_middleware, create_middleware_factory};
use crate::parse::{get_middleware_error_ident, get_vtype};

#[proc_macro_derive(HasThreadContext, attributes(vtype))]
pub fn derive_has_thread_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let vtype = get_vtype(input.attrs).unwrap();

    let ident = input.ident;
    let scope_context_type = Ident::new(&format!("__{}_ScopeContext", ident), Span::call_site());
    let scope_context_ident = Ident::new(&format!("__SCOPED_{}", ident).to_uppercase(), Span::call_site());


    (quote! {
        type #scope_context_type = std::collections::HashMap<std::thread::ThreadId, #vtype>;
        static mut #scope_context_ident: std::cell::OnceCell<#scope_context_type> = std::cell::OnceCell::new();

        impl actix_scoped::HasThreadContext for #ident {
            type Value = #vtype;
            unsafe fn thread_context_raw<'a>() -> &'a mut std::cell::OnceCell<std::collections::HashMap<std::thread::ThreadId, Self::Value>> {
                &mut #scope_context_ident
            }
        }
    }).into()
}

#[proc_macro_derive(Scoped, attributes(middleware))]
pub fn derive_scoped(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let middleware_error_ident = get_middleware_error_ident(input.attrs).unwrap();

    let ident = input.ident;
    let factory_ident = Ident::new(&format!("{}MiddlewareFactory", ident), Span::call_site());
    let middleware_ident = Ident::new(&format!("{}Middleware", ident), Span::call_site());

    let factory = create_middleware_factory(ident.clone(), factory_ident.clone(), middleware_ident.clone());
    let middleware = create_middleware(ident.clone(), middleware_error_ident.clone(), middleware_ident.clone());
    let from_request = create_from_request(ident.clone(), middleware_error_ident.clone());
    (quote! {
        impl actix_scoped::Scoped for #ident {}

        #factory
        #middleware
        #from_request

    }).into()
}


