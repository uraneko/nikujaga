use proc_macro2::Span;
use syn::Ident;

use crate::input::AliasScope;
use crate::input::Flag;
use crate::input::{AliasRule, CommandRule};

// TODO
// removal of duplicate aliases
// removal of invalid aliases (rules for tokens that dont exist)
pub struct AliasGenerator<'a> {
    cmd: &'a [CommandRule],
    alias: &'a mut Vec<AliasRule>,
}

impl<'a> AliasGenerator<'a> {
    pub fn new(alias: &'a mut Vec<AliasRule>, cmd: &'a [CommandRule]) -> Self {
        Self { alias, cmd }
    }

    pub fn generate_aliases(&mut self) {
        // get rid of invalid aliases
        // ie. aliases written for tokens that dont exist in any command rule
        self.resolve_hand_written();

        self.cmd.into_iter().for_each(|cr| {
            let space = space_alias(&self.alias, cr);
            let op = op_alias(&self.alias, cr);
            let flags = flags_aliases(&self.alias, cr);

            if let Some(ar) = space {
                self.push_alias(ar);
            }
            if let Some(ar) = op {
                self.push_alias(ar);
            }
            if let Some(ars) = flags {
                self.extend_alias(ars.into_iter());
            }
        });
    }

    // NOTE use this before generate_aliases to
    // remove invalid aliases and duplicates
    fn resolve_hand_written(&mut self) {
        let alias = std::mem::take(self.alias);
        self.alias
            .extend(alias.into_iter().filter(|a| match a.scope() {
                AliasScope::S => self.cmd.iter().any(|cr| cr.space() == a.token()),
                AliasScope::O => self.cmd.iter().any(|cr| cr.op() == a.token()),
                AliasScope::F => self.cmd.iter().any(|cr| {
                    let Some(flags) = cr.flags_idents() else {
                        return false;
                    };
                    flags.contains(&a.token())
                }),
            }));
    }

    fn push_alias(&mut self, ar: AliasRule) {
        self.alias.push(ar);
    }

    fn extend_alias<T: Iterator<Item = AliasRule>>(&mut self, ars: T) {
        self.alias.extend(ars);
    }
}

fn alias_exists(alias: &[AliasRule], scope: &AliasScope, token: &Ident) -> bool {
    alias
        .into_iter()
        .find(|ar| ar.scope() == scope && ar.token() == token)
        .is_some()
}

fn make_alias(scoped: AliasScope, token: Ident) -> Option<AliasRule> {
    let s = token.to_string();
    let len = s.len();
    let alen = if len <= 3 {
        0
    } else if scoped.is_space() {
        4
    } else {
        1
    };

    if alen > 0 {
        return Some(AliasRule::new(
            scoped,
            token,
            Ident::new(&s[..alen], Span::call_site()),
        ));
    }

    None
}

fn space_alias(alias: &[AliasRule], cr: &CommandRule) -> Option<AliasRule> {
    if !cr.contains_space() {
        return None;
    }
    let (scope, token) = (AliasScope::S, cr.space_cloned());
    if alias_exists(alias, &scope, &token) {
        return None;
    }

    make_alias(scope, token)
}

fn op_alias(alias: &[AliasRule], cr: &CommandRule) -> Option<AliasRule> {
    if !cr.contains_op() {
        return None;
    }
    let (scope, token) = (AliasScope::O, cr.op_cloned());
    if alias_exists(alias, &scope, &token) {
        return None;
    }

    make_alias(scope, token)
}

fn flag_alias(f: &Flag) -> Option<AliasRule> {
    make_alias(AliasScope::F, f.ident().clone())
}

use std::collections::HashSet;

fn flags_aliases(alias: &[AliasRule], cr: &CommandRule) -> Option<HashSet<AliasRule>> {
    if !cr.contains_flags() {
        return None;
    }
    let (scope, tokens) = (AliasScope::F, cr.flags()?);

    Some(
        tokens
            .into_iter()
            .map(|f| flag_alias(f))
            .filter(|ar| ar.is_some())
            .map(|ar| ar.unwrap())
            .collect::<HashSet<AliasRule>>(),
    )
}
