use proc_macro2::Ident;
use syn::{Attribute, Meta, Path};

pub fn attr_name(attr: &Attribute) -> Option<Path> {
    if let Meta::List(list) = &attr.meta {
        Some(list.path.clone())
    } else {
        None
    }
}