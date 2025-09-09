use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream as TS2};
use syn::Ident;
use syn::parse::{Parse, ParseStream, Result as PRes};

use super::scope::{Context, Scope};
use super::{AliasScope, Flag};
use crate::help::read_help;
use crate::output::AliasGenerator;
use crate::output::TypeTree;
use crate::output::{AliasLookup, TokenizedCommand};

pub mod alias;
pub mod command;
pub mod transform;

pub use alias::AliasRule;
pub use command::{CommandRule, ExpandingCommandRule};
pub use transform::TransformRule;

#[derive(Debug, Default)]
pub struct RulesUnresolved {
    alias: Vec<AliasRule>,
    cmd: Vec<ExpandingCommandRule>,
    trnsf: Vec<TransformRule>,
}

impl RulesUnresolved {
    pub fn commands_resolution(self, root_name: &str, ignore_nc: bool) -> Rules {
        Rules {
            alias: self.alias,
            trnsf: self.trnsf,
            // WARN too much collect/clone between this bit of code and
            // the expand method
            cmd: self
                .cmd
                .into_iter()
                .map(|mut exp| {
                    exp.resolve_naming_conventions(ignore_nc);
                    exp.resolve_direct_scopes(root_name);

                    exp.expand()
                })
                .flatten()
                // here we do operations naming conflicts resolutions
                // here we enforce naming conventions if their flag is on
                .collect::<HashSet<CommandRule>>()
                .into_iter()
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Rules {
    alias: Vec<AliasRule>,
    cmd: Vec<CommandRule>,
    trnsf: Vec<TransformRule>,
}

impl Rules {
    pub fn cmd_ref(&self) -> &[CommandRule] {
        &self.cmd
    }

    pub fn cmd_mut(&mut self) -> &mut Vec<CommandRule> {
        &mut self.cmd
    }

    pub fn alias_ref(&self) -> &[AliasRule] {
        &self.alias
    }

    pub fn alias_mut(&mut self) -> &mut Vec<AliasRule> {
        &mut self.alias
    }
}

impl Rules {
    #[deprecated]
    pub fn resolve_operations_naming_conflicts(&mut self) {
        // take the command rules
        let mut cmd = std::mem::take(&mut self.cmd);
        // declare needed variables
        let mut map: HashMap<u64, Vec<CommandRule>> = HashMap::new();
        let mut iter = cmd.into_iter();

        // group command rules by scope hash; i.e., by scope op equality
        while let Some(cr) = iter.next() {
            let scope_hash = cr.scope_hash();
            if let Some(ss) = map.get_mut(&scope_hash) {
                ss.push(cr);
            } else {
                map.insert(scope_hash, vec![cr]);
            }
        }

        // iterate over scoped groups
        let mut iter = map.into_values().into_iter();
        // check context equality
        // and change op names when required
        while let Some(group) = iter.next() {
            group.iter().map(|cr| (cr.space(), cr.op()));
            // for now we simply stick to checking if all contexts are equal
            // if so then nothing is done
            // otherwise we prefix all operations names with their spaces
            if group.len() == 1 || group.iter().all(|cr| cr.context() == group[0].context()) {
                self.cmd.extend(group);
            } else {
                self.cmd.extend(group.into_iter().map(|mut cr| {
                    cr.prefix_op();
                    cr
                }))
            }
        }
    }
}

impl Rules {
    pub fn inject_version_help(&mut self, root_name: &str) {
        let i = Ident::new(root_name, Span::call_site());
        self.cmd
            .extend([CommandRule::help(i.clone()), CommandRule::version(i)]);
    }

    pub fn alias_generator(&mut self, auto_alias: bool) {
        if !auto_alias {
            return;
        }

        let mut alias = std::mem::take(self.alias_mut());
        let mut alias_gen = AliasGenerator::new(&mut alias, self.cmd_ref());
        alias_gen.generate_aliases();

        self.alias = std::mem::take(&mut alias);
    }

    pub fn cmds_tokenizer(&self) -> Vec<TokenizedCommand> {
        self.cmd
            .iter()
            .map(|cmd| AliasLookup::new(cmd, &self.alias))
            .map(|al| al.lookup())
            .collect()
    }

    pub fn type_tree_renderer(&self, root_name: &str, derives: &[Ident]) -> TS2 {
        TypeTree::new(self.cmds_tokenizer(), root_name).render(derives)
    }

    pub fn root_type_help_implementor(&self, root_name: &str) -> TS2 {
        let toml = read_help();

        TypeTree::new(self.cmds_tokenizer(), root_name).help(toml)
    }
}

impl Parse for RulesUnresolved {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let mut alias = vec![];
        let mut cmd = vec![];
        let mut trnsf = vec![];
        while !stream.is_empty() {
            // TODO this could be better handled
            match stream.fork().parse::<Ident>()? {
                i if i == Ident::new("c", Span::call_site()) => {
                    cmd.push(ExpandingCommandRule::parse(stream)?)
                }

                i if i == Ident::new("t", Span::call_site()) => {
                    unimplemented!("transform rules have not been implemented yet")
                }
                i if [
                    Ident::new("s", Span::call_site()),
                    Ident::new("o", Span::call_site()),
                    Ident::new("f", Span::call_site()),
                ]
                .contains(&i) =>
                {
                    alias.push(AliasRule::parse(stream)?)
                }
                val => {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("expected c, t, s, o or f ident, got {}", val),
                    ));
                }
            }
        }

        Ok(Self { alias, cmd, trnsf })
    }
}
