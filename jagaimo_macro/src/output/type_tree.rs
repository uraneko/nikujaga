use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::{Ident, Type};

use super::{AliasedToken, TokenizedCommand};
use crate::help::{ExtractHelp, Extractor, Help, OpHelp, RootHelp, SpaceHelp};

use toml::Table;

// TODO
// impl transform rules so this can be done
// any command containing --help
// can be transformed from
// cli space op flags params --help
// into
// cli help space op
// the parser has no awareness of any flag named --help
// but the transform rules are applied before the command tokens are passed to the parser
// so it only sees the help operation

// TODO
// aliases are only needed to be passed on to the Help derive as a list attr
// they dont need to exist before or after that

#[derive(Debug, Clone)]
pub struct TypeTree<'a> {
    root: RootType<'a>,
}

// WARN space and op names are always there because every command needs a space and op names/ids for
// its states to be reprensented in the type tree
//  but when figuring the space and op of a command
//  the valid check is to use is_direct method on scope tokens
impl<'a> TypeTree<'a> {
    pub fn new(tcmd: Vec<TokenizedCommand<'a>>, root_name: &str) -> Self {
        let mut iter = tcmd.into_iter();
        let mut spaces: HashMap<&Ident, SpaceType<'_>> = HashMap::new();
        while let Some(cmd) = iter.next() {
            let space = cmd.space_cloned();
            let ident = space.ident().unwrap();

            if !spaces.contains(ident) {
                spaces.register(space);
            }

            let mut op = OpType::new(cmd.op_cloned());

            if let Some(flags) = cmd.flags_cloned() {
                flags.into_iter().for_each(|f| op.insert_flag(f))
            };

            if let Some(params) = cmd.params_cloned() {
                op.set_params(params);
            }

            // This inserts a full op type into a space with ident Ident
            spaces.update(ident, op);
        }

        let root_ident = Ident::new(root_name, Span::call_site());

        let root_ops = spaces.remove(&root_ident);
        let root_direct_op = root_ops.clone().map(|spc| spc.direct_op);
        let root_ops = root_ops.map(|spc| spc.ops);

        let root_spaces = spaces.into_values().collect::<Vec<SpaceType>>();

        let root = RootType {
            ident: root_ident,
            spaces: root_spaces,
            ops: root_ops.unwrap_or(Vec::new()),
            direct_op: root_direct_op.unwrap_or(None),
        };

        Self { root }
    }

    // calls all subordinate type.help methods
    // returns root type Help trait impl quote
    pub fn help(self, toml: Table) -> TS2 {
        let root = self.root.help(&toml);
        let ri = &self.root.ident;
        let root = quote! { [#ri, ""] => #root };
        let rops = self
            .root
            .ops
            .into_iter()
            .map(|op| (ri, op.token.ident().unwrap(), op.help(Some(ri), &toml)))
            .map(|(i, o, h)| quote! { [#i, #o] => #h });

        let spaces = self
            .root
            .spaces
            .clone()
            .into_iter()
            .map(|s| (s.token.ident().unwrap(), s.help(&toml)))
            .map(|(i, h)| quote! { [#i, ""] => #h });

        let ops = self
            .root
            .spaces
            .into_iter()
            .map(|s| {
                let tom = toml.clone();
                let i = s.token.ident();
                s.ops
                    .into_iter()
                    .map(move |op| (i.unwrap(), op.token.ident().unwrap(), op.help(i, &tom)))
                    .map(|(s, o, h)| quote! { [#s, #o] => #h })
            })
            .flatten();
        let arms = [root].into_iter().chain(rops).chain(spaces).chain(ops);

        quote! {
            impl Help for #ri {
                fn help(self, space: &str, op: &str) -> String {
                    match [space, op] {
                        #(#arms,)*
                        [s, o] => format("got unrecognized namespace or operation; {}, {}", s, o),
                    }
                }
            }
        }
    }

    pub fn render(self, derives: &[Ident]) -> TS2 {
        let root = self.root.render(derives);

        quote! {
            #root
        }
    }
}

trait TypeTreeExt<'a, T, C>
where
    T: 'a,
    C: 'a,
{
    fn contains(&self, ident: &Ident) -> bool;

    // it should be an error to attempt register a new
    // value only to find that it already existed in the map
    //
    // this function only creates new space types and sets their idents
    // pushing any variants to the SpaceType is out of the scope of this function
    fn register(&mut self, ident: AliasedToken<'a>);

    // it should be an error to attempt an update to an existing
    // value only to find that it didnt already exist in the map
    //
    // this function only inserts new variants into the given ident's
    // spacetype
    fn update(&mut self, ident: &Ident, variant: C);
}

impl<'a> TypeTreeExt<'a, SpaceType<'a>, OpType<'a>> for HashMap<&'a Ident, SpaceType<'a>> {
    fn contains(&self, ident: &Ident) -> bool {
        self.contains_key(ident)
    }

    fn register(&mut self, token: AliasedToken<'a>) {
        let i = token.ident().unwrap();
        _ = self.insert(
            i,
            SpaceType {
                token,
                direct_op: None,
                ops: Vec::new(),
            },
        );
    }

    fn update(&mut self, ident: &Ident, op: OpType<'a>) {
        self.get_mut(ident).map(|st| {
            if op
                .token
                .ident()
                .map(|i| i.to_string().ends_with("DirectOp"))
                .unwrap()
            {
                st.set_direct(op);
            } else {
                st.insert(op);
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootType<'a> {
    ident: Ident,
    spaces: Vec<SpaceType<'a>>,
    ops: Vec<OpType<'a>>,
    direct_op: Option<OpType<'a>>,
}

fn generate_derives(d: &[Ident]) -> Option<TS2> {
    if d.is_empty() {
        None
    } else {
        Some(quote! {
            # [derive( #(#d,)* )]
        })
    }
}

impl RootType<'_> {
    // generates the entire tree's help data
    // then implements the Help trait on the root type
    fn help(&self, toml: &Table) -> String {
        let extr = Extractor::new(Some(&self.ident), None, toml);
        let help: RootHelp = extr.extract(
            self.direct_op
                .as_ref()
                .map(|o| o.params.is_some())
                .unwrap_or(false),
            self.direct_op.is_some(),
        );

        help.fmt()
    }

    // renders the type tree as rust structs and enums
    fn render(self, derives: &[Ident]) -> TS2 {
        let derive = generate_derives(derives);
        let ident = self.ident;
        let count =
            self.spaces.len() + self.ops.len() + if self.direct_op.is_some() { 1 } else { 0 };

        let root = if count == 1 {
            if let Some(op) = self.direct_op {
                let op = op.render_fields();

                return quote! {
                    #derive
                    struct #ident {
                        #op
                    }
                };
            } else if self.spaces.len() == 1 {
                let [module, field] = self
                    .spaces
                    .into_iter()
                    .next()
                    .map(|s| {
                        [s.clone().render(derives), {
                            let module = s.module_name();
                            let ident = s.token.ident().unwrap();

                            quote! {
                                #module: #module :: #ident
                            }
                        }]
                    })
                    .unwrap();

                return quote! {
                    #derive
                    pub struct #ident {
                        #field
                    }

                    #module
                };
            } else {
                let op = self
                    .ops
                    .into_iter()
                    .next()
                    .map(|op| op.render_fields())
                    .unwrap();

                return quote! {
                    #derive
                    pub struct #ident {
                        #op
                    }
                };
            }
        } else {
            let direct_variant = self
                .direct_op
                .clone()
                .map(|op| {
                    (
                        {
                            Ident::new(
                                &op.token
                                    .ident()
                                    .unwrap()
                                    .to_string()
                                    .replace("DirectOp", ""),
                                Span::call_site(),
                            )
                        },
                        op.render_fields(),
                    )
                })
                .map(|(i, f)| quote! { #i { #f }});
            let direct = self.direct_op.map(|op| op.render());

            let op_variants = self
                .ops
                .clone()
                .into_iter()
                .map(|op| op.render())
                .map(|op| quote! { #op });
            let ops = self.ops.into_iter().map(|op| op.render());

            let space_variants = self
                .spaces
                .clone()
                .into_iter()
                .map(|scp| scp.render_variant());
            let spaces = self.spaces.into_iter().map(|spc| spc.render(derives));

            return quote! {
                #derive
                pub enum #ident {
                    #(#space_variants,)*
                    #(#op_variants,)*
                    #direct_variant
                }

                #(#spaces)*
            };
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceType<'a> {
    token: AliasedToken<'a>,
    ops: Vec<OpType<'a>>,
    direct_op: Option<OpType<'a>>,
}

impl<'a> SpaceType<'a> {
    fn insert(&mut self, op: OpType<'a>) {
        _ = self.ops.push(op);
    }

    fn set_direct(&mut self, op: OpType<'a>) {
        self.direct_op = Some(op);
    }

    fn is_space_direct(&self) -> bool {
        self.token.is_direct()
    }

    fn is_op_direct(&self) -> bool {
        self.token.is_root() && self.direct_op.is_some()
    }

    fn module_name(&self) -> Ident {
        let s = self.token.ident().unwrap().to_string();
        Ident::new(
            &s.chars()
                .enumerate()
                .map(|(i, c)| {
                    if c.is_ascii_uppercase() {
                        if i == 0 {
                            String::from(c.to_ascii_lowercase())
                        } else {
                            let mut s = String::from("_");
                            s.push(c.to_ascii_lowercase());
                            s
                        }
                    } else {
                        String::from(c)
                    }
                })
                .fold(String::new(), |acc, s| acc + &s),
            Span::call_site(),
        )
    }

    fn has_sole_op(&self) -> bool {
        self.ops.len() + if self.direct_op.is_some() { 1 } else { 0 } == 1
    }

    fn render_variant(self) -> TS2 {
        let module_name = self.module_name();
        let ident = self.token.ident().unwrap().clone();

        if self.has_sole_op() {
            let op = if let Some(op) = self.direct_op {
                op
            } else {
                self.ops.into_iter().next().unwrap()
            };
            let opi = if op.token.is_direct_op() {
                Ident::new(
                    &ident.to_string().replace("DirectOp", ""),
                    Span::call_site(),
                )
            } else {
                Ident::new(
                    &(ident.to_string() + &op.token.ident().unwrap().to_string()),
                    Span::call_site(),
                )
            };

            let fields = op.render_fields();

            return quote! {
                #opi { #fields }
            };
        } else {
            let ident = self.token.ident().unwrap().clone();
            return quote! {
                #ident ( #ident )
            };
        }
    }

    // returns the help match arm quote for this space
    fn help(&self, toml: &Table) -> String {
        let extr = Extractor::new(self.token.ident(), None, toml);
        let help: SpaceHelp = extr.extract(
            self.direct_op
                .as_ref()
                .map(|o| o.params.is_some())
                .unwrap_or(false),
            self.direct_op.is_some(),
        );

        help.fmt()
    }

    fn render(self, derives: &[Ident]) -> TS2 {
        let derives = generate_derives(derives);
        let ident = self.token.ident().unwrap();
        let mod_ident = self.module_name();

        let op_count = if self.direct_op.is_some() { 1 } else { 0 } + self.ops.len();
        if op_count == 1 {
            return quote! {};
        };
        let space_type = if op_count == 1 {
            let (op, ident) = if let Some(op) = self.direct_op {
                (op, ident.clone())
            } else {
                let op = self.ops.into_iter().next().unwrap();
                let ident = Ident::new(
                    &(ident.to_string() + &op.token.ident().unwrap().to_string()),
                    Span::call_site(),
                );
                (op, ident)
            };
            let fields = op.render_fields();
            // TODO let ident = ident + op ident ;
            quote! {
                #derives
                pub struct #ident {
                    #fields
                }
            }
        } else {
            let direct_variant = self
                .direct_op
                .clone()
                .map(|op| op.token.ident())
                .map(|i| quote! { #i ( #i )});
            let direct = self.direct_op.map(|op| op.render());

            let op_variants = self
                .ops
                .clone()
                .into_iter()
                .map(|op| op.token.ident())
                .map(|i| quote! { #i ( #i ) });
            let ops = self.ops.into_iter().map(|op| op.render());

            quote! {
                #derives
                pub enum #ident {
                    #(#ops,)*
                    #direct
                }
            }
        };

        quote! {
            #space_type
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpType<'a> {
    token: AliasedToken<'a>,
    fields: Vec<AliasedToken<'a>>,
    params: Option<AliasedToken<'a>>,
}

impl<'a> OpType<'a> {
    fn new(atok: AliasedToken<'a>) -> Self {
        Self {
            token: atok,
            fields: Vec::new(),
            params: None,
        }
    }

    fn insert_flag(&mut self, field: AliasedToken<'a>) {
        _ = self.fields.push(field);
    }

    fn set_params(&mut self, params: AliasedToken<'a>) {
        _ = self.params = Some(params);
    }

    fn render_fields(self) -> TS2 {
        let fields = self.fields.into_iter().map(|atok| {
            let f = atok.flag().unwrap();
            let ident = f.ident();
            let fallback = Type::Verbatim("bool".parse().unwrap());
            let ty = f.ty().unwrap_or(&fallback);

            quote! { #ident: #ty  }
        });
        let params = self.params.map(|atok| {
            let ty = atok.ty();
            quote! { params: #ty }
        });

        quote! {
            #(#fields,)*
            #params
        }
    }

    // returns the help match arm for this op
    fn help(&self, s: Option<&Ident>, toml: &Table) -> String {
        let o = self.token.ident();
        let extr = Extractor::new(s, o, toml);
        let help: OpHelp = extr.extract(self.params.is_some(), false);

        help.fmt()
    }

    fn render(self) -> TS2 {
        let ident = self.token.ident();
        let len = if self.params.is_some() { 1 } else { 0 } + self.fields.len();
        let fields = self.fields.into_iter().map(|atok| {
            let f = atok.flag().unwrap();
            let ident = f.ident();
            let fallback = Type::Verbatim("bool".parse().unwrap());
            let ty = f.ty().unwrap_or(&fallback);

            quote! { #ident: #ty  }
        });
        let params = self.params.map(|atok| {
            let ty = atok.ty();
            quote! { params: #ty }
        });

        if len == 0 {
            quote! { #ident }
        } else {
            quote! {
                #ident {
                    #(#fields,)*
                    #params
                }
            }
        }
    }
}
