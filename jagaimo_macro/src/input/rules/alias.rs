use std::hash::{Hash, Hasher};

use syn::Ident;
use syn::Token;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::{braced, parenthesized};

use super::AliasScope;

#[derive(Debug, Clone, Eq)]
pub struct AliasRule {
    scoped: AliasScope,
    token: Ident,
    alias: Ident,
}

impl Hash for AliasRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.scoped.hash(state);
        self.alias.hash(state);
    }
}

impl PartialEq for AliasRule {
    fn eq(&self, other: &Self) -> bool {
        self.alias == other.alias && self.scoped == other.scoped
    }
}

impl AliasRule {
    pub fn new(scoped: AliasScope, token: Ident, alias: Ident) -> Self {
        Self {
            scoped,
            token,
            alias,
        }
    }

    pub fn scope(&self) -> &AliasScope {
        &self.scoped
    }

    pub fn token(&self) -> &Ident {
        &self.token
    }

    pub fn alias(&self) -> &Ident {
        &self.alias
    }
}

impl Parse for AliasRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let content;
        let scoped = Ident::parse(stream)?.try_into()?;
        _ = parenthesized!(content in stream);
        let token = Ident::parse(&content)?;
        _ = <Token![=]>::parse(&stream)?;
        let alias = Ident::parse(&stream)?;

        Ok(Self {
            token,
            alias,
            scoped,
        })
    }
}

impl std::fmt::Display for AliasRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{} -> {}", self.scope(), self.token(), self.alias)
    }
}
