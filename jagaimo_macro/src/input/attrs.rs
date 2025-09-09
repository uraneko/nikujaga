use proc_macro2::Span;
use syn::Token;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::{Ident, Lit};
use syn::{bracketed, parenthesized};

use super::enforce_str_nc;
use resolve_calling_crate::ResolveCrate;

#[derive(Debug)]
pub struct Attrs {
    root_name: String,
    help: bool,
    version: bool,
    nu_completions: bool,
    fish_completions: bool,
    ignore_naming_conventions: bool,
    auto_alias: bool,
    derives: Vec<Ident>,
}

impl Attrs {
    pub fn derives(&self) -> &[Ident] {
        &self.derives
    }

    pub fn fish_cmp(&self) -> bool {
        self.fish_completions
    }

    pub fn nu_cmp(&self) -> bool {
        self.nu_completions
    }

    pub fn root_name(&self) -> &str {
        &self.root_name
    }

    pub fn auto_alias(&self) -> bool {
        self.auto_alias
    }

    pub fn help(&self) -> bool {
        self.help
    }

    pub fn version(&self) -> bool {
        self.version
    }

    pub fn ignore_nc(&self) -> bool {
        self.ignore_naming_conventions
    }
}

impl Default for Attrs {
    fn default() -> Self {
        Self {
            root_name: ResolveCrate::new().read_manifest().crate_name().to_string(),
            help: true,
            version: true,
            nu_completions: false,
            fish_completions: false,
            ignore_naming_conventions: false,
            auto_alias: true,
            derives: Self::default_derives(),
        }
    }
}

impl Attrs {
    fn default_derives() -> Vec<Ident> {
        vec![
            Ident::new("Debug", Span::call_site()),
            Ident::new("PartialEq", Span::call_site()),
            Ident::new("Clone", Span::call_site()),
        ]
    }
}

impl Parse for Attrs {
    fn parse(stream: ParseStream) -> PRes<Self> {
        let mut attrs = Attrs::default();
        // NOTE this should consume ZERO tokens if the user provided no attributes

        if !stream.peek(Token![#]) {
            return Ok(attrs);
        }

        _ = <Token![#]>::parse(stream)?;
        let content;
        let _brackets = bracketed!(content in stream);
        // WARN doesnt work
        // cus not all attrs are boolean
        while content.peek(Ident::peek_any) {
            match &Ident::parse(&content)?.to_string()[..] {
                // "fish_cmp"
                // | "nu_cmp"
                // | "ignore_naming_conventions"
                // | "no_help"
                // | "no_version" => unimplemented!(),
                "root_name" => {
                    _ = <Token![=]>::parse(&content)?;
                    let name = Lit::parse(&content)?;
                    let Lit::Str(ls) = name else {
                        unreachable!("root_name attr has to take a single str lit")
                    };

                    attrs.root_name = ls.value();
                }
                "fish_cmp" => attrs.fish_completions = true,
                "nu_cmp" => attrs.nu_completions = true,
                "no_help" => attrs.help = false,
                "no_version" => attrs.version = false,
                "ignore_naming_conventions" => attrs.ignore_naming_conventions = true,
                "no_auto_alias" => attrs.auto_alias = false,
                "derives" => {
                    let derives;
                    let _parens = parenthesized!(derives in content);
                    attrs.derives = std::mem::take(
                        &mut derives
                            .parse_terminated(Ident::parse, Token![,])?
                            .into_iter()
                            .collect(),
                    );
                }
                val => panic!("unrecognized mock attribute {}", val),
            }

            if !content.is_empty() {
                _ = <Token![,]>::parse(&content)?;
            }
        }
        // TODO dont capitalize any type name when this flag is on
        if !attrs.ignore_naming_conventions {
            // TODO all type tree type idents need this
            enforce_str_nc(&mut attrs.root_name);
        }

        Ok(attrs)
    }
}
