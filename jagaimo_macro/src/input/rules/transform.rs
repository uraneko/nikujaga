use std::hash::{Hash, Hasher};

use syn::parse::{Parse, ParseStream, Result as PRes};

#[derive(Debug, Default)]
pub struct TransformRule {}

impl Parse for TransformRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        todo!()
    }
}
