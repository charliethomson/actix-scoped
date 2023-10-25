use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use syn::{Attribute, Meta};
use crate::util::attr_name;

fn find_attr_by_name<'a>(attrs: &'a Vec<Attribute>, name: &'static str) -> Option<&'a Attribute> {
    for attr in attrs {
        let Some(path) = attr_name(attr) else { continue; };
        let Some(ident) = path.get_ident() else { continue; };
        if ident.to_string() == name { return Some(attr); }
    }
    return None;
}

pub fn get_vtype(attrs: Vec<Attribute>) -> Option<TokenStream> {
    let attr = find_attr_by_name(&attrs, "vtype")?;
    if let Meta::List(list) = &attr.meta {
        Some(list.tokens.clone())
    } else {
        None
    }
}

pub fn get_middleware_error_ident(attrs: Vec<Attribute>) -> Option<TokenStream> {
    let attr = find_attr_by_name(&attrs, "middleware")?;
    if let Meta::List(list) = &attr.meta {
        Some(list.tokens.clone().into_iter().skip(2).collect())
    } else {
        None
    }
}
