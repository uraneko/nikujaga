use proc_macro2::Span;
use syn::Ident;
use syn::parse::{Parse, ParseStream, Result as PRes};

pub mod attrs;
pub mod flags;
pub mod rules;
pub mod scope;

pub use attrs::Attrs;
pub use flags::Flag;
pub use rules::{AliasRule, CommandRule, RulesUnresolved};
pub use scope::AliasScope;

// this swaps the passed ident with a changed one
pub fn to_enforced_ident_nc(ident: &mut Ident) {
    let mut name = ident.to_string();
    name.replace_range(
        0..1,
        &name.get(0..1).map(|s| s.to_ascii_uppercase()).unwrap(),
    );

    while let Some(idx) = name.find('_') {
        let next = name.get(idx + 1..idx + 2);
        if let Some(s) = next {
            name.replace_range(idx..idx + 2, &s.to_ascii_uppercase());
        }
    }

    std::mem::swap(ident, &mut Ident::new(&name, Span::call_site()))
}

// this mutates the passed string in place
pub fn enforce_str_nc(name: &mut String) {
    name.replace_range(
        0..1,
        &name.get(0..1).map(|s| s.to_ascii_uppercase()).unwrap(),
    );

    while let Some(idx) = name.find('_') {
        let next = name.get(idx + 1..idx + 2);
        if let Some(s) = next {
            name.replace_range(idx..idx + 2, &s.to_ascii_uppercase());
        }
    }
}

pub type MacroInput = (Attrs, RulesUnresolved);

pub fn parse_attrs_rules(stream: ParseStream) -> PRes<MacroInput> {
    Ok((Attrs::parse(stream)?, RulesUnresolved::parse(stream)?))
}
