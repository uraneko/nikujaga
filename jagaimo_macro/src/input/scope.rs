use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Parser, Result as PRes};
use syn::{Ident, Type};

use super::CommandRule;
use super::Flag;

// #[derive(Debug)]
// pub enum Scope {
//     Root,
//     Space(Ident),
//     SpaceOperation { space: Ident, op: Ident },
//     Operation(Ident),
// }

#[derive(Debug, Eq, Clone, PartialOrd, Ord)]
pub struct Scope<'a> {
    space: &'a Ident,
    op: &'a Ident,
}

impl Scope<'_> {
    pub fn into_hash(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        hasher.finish()
    }
}

impl<'a> From<&'a CommandRule> for Scope<'a> {
    fn from(cr: &'a CommandRule) -> Self {
        Self {
            space: cr.space(),
            op: cr.op(),
        }
    }
}

// WARN the following 2 impls
// make Scope behave in truly non intuitive ways as a hashmap key
// the reason for doing this
// is that there is no point in checking if scopes are equal
// since duplicate scopes are discarded beforehand
// what we really care about is operation equality
// which may lead to type tree naming conflicts
impl Hash for Scope<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.op.hash(state);
    }
}

impl PartialEq for Scope<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op
    }
}

#[derive(Debug, Default, Eq, Clone)]
pub struct Context<'a> {
    flags: Option<&'a [Flag]>,
    params: Option<&'a Type>,
}

// WARN
// these following 2 impls are conventional/intuitive
// but are needed because Flag's hash/partialeq impls
// are not intuitive/have unexpected behaviour
impl Hash for Context<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(flags) = self.flags {
            flags.iter().for_each(|f| {
                let fident = f.ident();
                let fty = f.ty();
                fident.hash(state);
                fty.hash(state);
            });
        } else {
            self.flags.hash(state)
        }

        self.params.hash(state);
    }
}

impl PartialEq for Context<'_> {
    fn eq(&self, other: &Self) -> bool {
        match [self.flags, other.flags] {
            [None, None] => self.params == other.params,
            [None, Some(_)] => false,
            [Some(_), None] => false,
            [Some(sflags), Some(oflags)] => {
                sflags.len() == oflags.len()
                    && std::iter::zip(sflags, oflags)
                        .all(|(sf, of)| sflags.contains(of) && oflags.contains(sf))
                    && self.params == other.params
            }
        }
    }
}

impl<'a> From<&'a CommandRule> for Context<'a> {
    fn from(cr: &'a CommandRule) -> Self {
        Self {
            flags: cr.flags(),
            params: cr.params(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum AliasScope {
    // a space
    S,
    // an operation
    O,
    // a flag
    F,
}

impl AliasScope {
    pub fn is_space(&self) -> bool {
        let Self::S = self else { return false };

        true
    }
}

impl TryFrom<Ident> for AliasScope {
    type Error = syn::Error;

    fn try_from(ident: Ident) -> PRes<Self> {
        match ident {
            i if i == Ident::new("s", Span::call_site()) => Ok(Self::S),
            i if i == Ident::new("o", Span::call_site()) => Ok(Self::O),
            i if i == Ident::new("f", Span::call_site()) => Ok(Self::F),
            _ => Err(syn::Error::new(
                Span::call_site(),
                "AliasScope try_from Ident takes 1 of s, o or f idents",
            )),
        }
    }
}
