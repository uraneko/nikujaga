use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use proc_macro2::Span;
use quote::ToTokens;
use syn::Token;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::token::Bracket;
use syn::{Ident, Type};
use syn::{braced, bracketed, parenthesized};

use super::Flag;
use crate::input::scope::{Context, Scope};
use crate::input::to_enforced_ident_nc;

// TODO
// deduplication of rules
#[derive(Debug, Clone, Eq)]
pub struct CommandRule {
    is_of_root: bool,
    space: Ident,
    op: Ident,
    flags: Option<Vec<Flag>>,
    params: Option<Type>,
}

impl CommandRule {
    pub fn help(root_ident: Ident) -> Self {
        Self {
            is_of_root: true,
            space: root_ident,
            op: Ident::new("Help", Span::call_site()),
            flags: None,
            params: Some(Type::Verbatim("Scope".parse().unwrap())),
        }
    }

    pub fn version(root_ident: Ident) -> Self {
        Self {
            is_of_root: true,
            space: root_ident,
            op: Ident::new("Version", Span::call_site()),
            flags: None,
            params: None,
        }
    }
}

// WARN 2 command rules that have the same scope (space and op)
// should be considered equal reagrdless of their context

impl Hash for CommandRule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.space.hash(state);
        self.op.hash(state);
    }
}

impl PartialEq for CommandRule {
    fn eq(&self, other: &Self) -> bool {
        self.space == other.space && self.op == other.op
    }
}

impl CommandRule {
    pub fn contains_space(&self) -> bool {
        !self.is_of_root
    }

    pub fn contains_op(&self) -> bool {
        is_direct_op(&self.op)
    }

    pub fn contains_flags(&self) -> bool {
        self.flags.is_some()
    }
}

impl CommandRule {
    pub fn space_cloned(&self) -> Ident {
        self.space.clone()
    }

    pub fn op_cloned(&self) -> Ident {
        self.op.clone()
    }

    pub fn space(&self) -> &Ident {
        &self.space
    }

    pub fn op(&self) -> &Ident {
        &self.op
    }

    pub fn scope_hash(&self) -> u64 {
        <&Self as Into<Scope>>::into(&self).into_hash()
    }

    pub fn context(&self) -> Context {
        self.into()
    }

    pub fn prefix_op(&mut self) {
        self.op = Ident::new(
            &{
                let s = format!("{}{}", self.space, self.op);

                s
            },
            Span::call_site(),
        );
    }

    pub fn flags(&self) -> Option<&[Flag]> {
        self.flags.as_ref().map(|flags| flags.as_slice())
    }

    pub fn flags_idents(&self) -> Option<Vec<&Ident>> {
        self.flags
            .as_ref()
            .map(|f| f.into_iter().map(|f| f.ident()).collect())
    }

    pub fn params(&self) -> Option<&Type> {
        self.params.as_ref()
    }
}

#[derive(Debug)]
pub struct ExpandingCommandRule {
    spaces: Vec<Ident>,
    ops: Vec<Ident>,
    flags: Vec<Flag>,
    params: Option<Type>,
}

fn extract_context_tokens(s: ParseStream) -> PRes<(Vec<Flag>, Option<Type>)> {
    let mut f = HashSet::new();
    let p = if s.peek(Token![<]) {
        _ = <Token![<]>::parse(s)?;
        let p = Type::parse(s)?;
        _ = <Token![>]>::parse(s)?;

        Some(p)
    } else {
        None
    };

    while !s.is_empty() {
        f.insert(Flag::parse(s)?);
    }

    Ok((f.into_iter().collect(), p))
}

pub fn extract_scope_tokens(s: ParseStream, scope: &str) -> PRes<Vec<Ident>> {
    // anonymous scope
    if s.peek(Bracket) {
        return Ok(vec![]);
    }
    // just because next is not a [
    // doesnt mean it would be an ident
    // but failing is such a case is intended behaviour

    if s.fork().parse::<Ident>()? == Ident::new(scope, Span::call_site()) {
        _ = Ident::parse(s)?;
        let scopes;
        _ = parenthesized!(scopes in s);
        // use punctuated instead
        return scopes
            .parse_terminated(Ident::parse, Token![,])
            .map(|p| p.into_iter().collect());
    }

    Ok(vec![])
}

// makes a new ident for a direct operation from a space ident
fn direct_from_ident(ident: &Ident) -> Ident {
    Ident::new(&(ident.to_string() + "DirectOp"), Span::call_site())
}

// makes a new ident for a direct operation from a space stringified ident
fn direct_from_str(s: &str) -> Ident {
    Ident::new(&(s.to_string() + "DirectOp"), Span::call_site())
}

// checks if an operation is a direct one
pub fn is_direct_op(i: &Ident) -> bool {
    i.to_string().ends_with("DirectOp")
}

// checks if an operation is direct to root
pub fn is_root_direct_op(i: &Ident, root_name: &str) -> bool {
    let s = i.to_string();
    s.starts_with(root_name) && s.ends_with("DirectOp")
}

impl ExpandingCommandRule {
    fn new(spaces: Vec<Ident>, ops: Vec<Ident>, flags: Vec<Flag>, params: Option<Type>) -> Self {
        Self {
            spaces,
            ops,
            flags,
            params,
        }
    }

    fn take_flags(&mut self) -> Option<Vec<Flag>> {
        if self.flags.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.flags))
        }
    }

    fn clone_flags(&self) -> Option<Vec<Flag>> {
        if self.flags.is_empty() {
            None
        } else {
            Some(self.flags.clone())
        }
    }

    fn clone_params(&self) -> Option<Type> {
        self.params.clone()
    }

    pub fn resolve_naming_conventions(&mut self, ignore_nc: bool) {
        if ignore_nc {
            return;
        }

        self.spaces.iter_mut().for_each(|s| {
            to_enforced_ident_nc(s);
        });
        self.ops.iter_mut().for_each(|s| {
            to_enforced_ident_nc(s);
        });
    }

    pub fn resolve_direct_scopes(&mut self, root_name: &str) {
        match [self.spaces.is_empty(), self.ops.is_empty()] {
            [false, false] => (),
            [true, true] => {
                self.spaces.push(Ident::new(root_name, Span::call_site()));
                self.ops.push(direct_from_str(root_name));
            }
            [true, false] => {
                self.spaces.push(Ident::new(root_name, Span::call_site()));
            }
            [false, true] => {
                self.spaces.iter().for_each(|s| {
                    self.ops.push(direct_from_ident(s));
                });
            }
        }
    }

    pub fn expand(mut self) -> Vec<CommandRule> {
        match [self.spaces.is_empty(), self.ops.is_empty()] {
            [true, true] => vec![CommandRule {
                is_of_root: true,
                flags: self.take_flags(),
                params: self.params,
                space: self.spaces.remove(0),
                op: self.ops.remove(0),
            }],

            [true, false] => {
                let ops = std::mem::take(&mut self.ops);
                ops.into_iter()
                    .map(|o| CommandRule {
                        op: o,
                        is_of_root: true,
                        space: self.spaces[0].clone(),
                        flags: self.clone_flags(),
                        params: self.clone_params(),
                    })
                    .collect()
            }

            [false, true] => {
                let spaces = std::mem::take(&mut self.spaces);
                spaces
                    .into_iter()
                    .map(|s| CommandRule {
                        space: s,
                        is_of_root: false,
                        op: self.ops[0].clone(),
                        params: self.clone_params(),
                        flags: self.clone_flags(),
                    })
                    .collect()
            }

            [false, false] => self
                .spaces
                .iter()
                .map(|s| {
                    self.ops.iter().map(|o| CommandRule {
                        space: s.clone(),
                        op: o.clone(),
                        is_of_root: false,
                        flags: self.clone_flags(),
                        params: self.clone_params(),
                    })
                })
                .flatten()
                .collect(),
        }
    }
}

impl Parse for ExpandingCommandRule {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let _rule_name = Ident::parse(stream)?;

        let content;
        _ = braced!(content in stream);
        // there is some scope

        let spaces = extract_scope_tokens(&content, "s")?;
        let ops = extract_scope_tokens(&content, "o")?;

        if !content.peek(Bracket) {
            return Ok(ExpandingCommandRule::new(spaces, ops, vec![], None));
        }

        let context;
        _ = bracketed!(context in content);
        let (flags, params) = extract_context_tokens(&context)?;

        Ok(ExpandingCommandRule::new(spaces, ops, flags, params))
    }
}

impl std::fmt::Display for CommandRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SPACE<{}> OPERATION<{}> PARAM<{}> FLAGS<{}>\n",
            self.space.to_string(),
            self.op.to_string(),
            self.params
                .as_ref()
                .map(|t| format!("{:?}", t.to_token_stream().to_string()))
                .unwrap_or("".into()),
            self.flags
                .as_ref()
                .map(|f| f
                    .into_iter()
                    .map(|f| f.to_string())
                    .fold(String::new(), |acc, f| acc + &f + " "))
                .unwrap_or("".into()),
        )
    }
}
