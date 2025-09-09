use syn::{Ident, Type};

use crate::input::{AliasRule, AliasScope, CommandRule, Flag};

macro_rules! aliased_token {
    ('f', $t: ident, $a: ident) => {
        AliasedToken::Flag {
            flag: $t,
            alias: Some($a),
        }
    };

    ($s: literal, $t: ident, $a: ident, $n: ident) => {
        match $s {
            's' => AliasedToken::Space {
                space: $t,
                alias: Some($a),
                is_root: $n,
            },
            'o' => AliasedToken::Operation {
                op: $t,
                alias: Some($a),
            },
            _ => panic!("macro expected 's' or 'o' as first token"),
        }
    };

    ('f', $t: ident) => {
        AliasedToken::Flag {
            flag: $t,
            alias: None,
        }
    };

    ($s: literal, $t: ident, $n: ident) => {
        match $s {
            's' => AliasedToken::Space {
                space: $t,
                alias: None,
                is_root: $n,
            },
            'o' => AliasedToken::Operation {
                op: $t,
                alias: None,
            },
            _ => panic!("macro expected 's', 'o' or 'f' as first token"),
        }
    };

    ($p: ident) => {
        AliasedToken::Params($p)
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct AliasLookup<'a> {
    cmd: &'a CommandRule,
    alias: &'a [AliasRule],
}

impl<'a, 'b> AliasLookup<'b>
where
    'a: 'b,
{
    // creates a new AliasLookup from the passed command rule and slice of alias rules
    pub fn new(cmd: &'a CommandRule, alias: &'a [AliasRule]) -> Self {
        Self { cmd, alias }
    }

    // returns self.cmd's space alias value if it exists
    fn space_aliased(&self) -> AliasedToken<'b> {
        // WARN altered for the sake expanding
        // if let Some(space) = self.cmd.space() { ...
        let space = self.cmd.space();
        let direct = !self.cmd.contains_space();
        let atok = self
            .alias
            .into_iter()
            .find(|al| al.scope() == &AliasScope::S && al.token() == space);
        if let Some(al) = atok {
            let al = al.alias();

            return aliased_token!('s', space, al, direct);
        }

        aliased_token!('s', space, direct)
    }

    // returns self.cmd's op alias value if it exists
    fn op_aliased(&self) -> AliasedToken<'b> {
        // WARN altered for the sake expanding
        // if let Some(op) = self.cmd.op() { ...
        let op = self.cmd.op();
        let direct = !self.cmd.contains_op();
        let atok = self
            .alias
            .into_iter()
            .find(|al| al.scope() == &AliasScope::O && al.token() == op);
        if let Some(al) = atok {
            let al = al.alias();

            return aliased_token!('o', op, al, direct);
        }

        aliased_token!('o', op, direct)
    }

    // returns self.cmd's flags alias values if at least one exists
    fn flags_aliased(&self) -> Option<Vec<AliasedToken<'b>>> {
        if let Some(flags) = self.cmd.flags() {
            return Some(
                flags
                    .into_iter()
                    .map(|f| {
                        let i = f.ident();
                        let al = self
                            .alias
                            .into_iter()
                            .find(|al| al.scope() == &AliasScope::F && i == al.token());

                        if al.is_some() {
                            let a = al.unwrap().alias();
                            aliased_token!('f', f, a)
                        } else {
                            aliased_token!('f', f)
                        }
                    })
                    .collect::<Vec<AliasedToken>>(),
            );
        }

        None
    }

    pub fn lookup(self) -> TokenizedCommand<'b> {
        let cmd = TokenizedCommand::new(
            self.space_aliased(),
            self.op_aliased(),
            self.flags_aliased(),
            self.cmd.params().map(|p| aliased_token!(p)),
        );

        cmd
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AliasedToken<'a> {
    Space {
        space: &'a Ident,
        alias: Option<&'a Ident>,
        is_root: bool,
    },

    Operation {
        op: &'a Ident,
        alias: Option<&'a Ident>,
    },

    Flag {
        flag: &'a Flag,
        alias: Option<&'a Ident>,
    },

    // this is not really an aliased token
    // but it is needed to be able to vectorize the tokenized command into a vec of tokens
    Params(&'a Type),
}

impl<'a> AliasedToken<'a> {
    pub fn new_op(ident: &'a Ident) -> Self {
        Self::Operation {
            op: ident,
            alias: None,
        }
    }

    pub fn is_space(&self) -> bool {
        let Self::Space { .. } = self else {
            return false;
        };

        true
    }

    pub fn is_root(&self) -> bool {
        let Self::Space { is_root, .. } = self else {
            return false;
        };

        *is_root
    }

    pub fn is_direct(&self) -> bool {
        match self {
            Self::Space { is_root, .. } => *is_root,
            Self::Operation { .. } => self.is_direct(),
            _ => false,
        }
    }

    pub fn is_op(&self) -> bool {
        let Self::Operation { .. } = self else {
            return false;
        };

        true
    }

    pub fn is_direct_op(&self) -> bool {
        let Self::Operation { op, .. } = self else {
            return false;
        };

        crate::input::rules::command::is_direct_op(op)
    }

    pub fn is_flag(&self) -> bool {
        let Self::Flag { .. } = self else {
            return false;
        };

        true
    }

    pub fn ident(&self) -> Option<&'a Ident> {
        match self {
            Self::Space { space, .. } => Some(space),
            Self::Operation { op, .. } => Some(op),
            _ => None,
        }
    }

    pub fn flag(&self) -> Option<&'a Flag> {
        let Self::Flag { flag, .. } = self else {
            return None;
        };

        Some(flag)
    }

    pub fn ty(&self) -> Option<&'a Type> {
        match self {
            Self::Flag { flag, .. } => flag.ty(),
            Self::Params(ty) => Some(ty),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenizedCommand<'a> {
    space: AliasedToken<'a>,
    op: AliasedToken<'a>,
    flags: Option<Vec<AliasedToken<'a>>>,
    params: Option<AliasedToken<'a>>,
}

impl TokenizedCommand<'_> {
    #[deprecated]
    pub fn is_space_op(&self) -> bool {
        !self.space.is_direct() && !self.op.is_direct()
    }

    #[deprecated]
    pub fn is_op(&self) -> bool {
        self.space.is_direct() && !self.op.is_direct()
    }

    #[deprecated]
    pub fn is_space(&self) -> bool {
        !self.space.is_direct() && self.op.is_direct()
    }
}

impl<'a> TokenizedCommand<'a> {
    pub fn new(
        space: AliasedToken<'a>,
        op: AliasedToken<'a>,
        flags: Option<Vec<AliasedToken<'a>>>,
        params: Option<AliasedToken<'a>>,
    ) -> Self {
        Self {
            space,
            op,
            flags,
            params,
        }
    }
    pub fn space_cloned(&self) -> AliasedToken<'a> {
        self.space.clone()
    }

    pub fn space(&self) -> &AliasedToken<'a> {
        &self.space
    }

    pub fn op(&self) -> &AliasedToken<'a> {
        &self.op
    }

    pub fn op_cloned(&self) -> AliasedToken<'a> {
        self.op.clone()
    }

    pub fn flags_cloned(&self) -> Option<Vec<AliasedToken<'a>>> {
        self.flags.clone()
    }

    pub fn params_cloned(&self) -> Option<AliasedToken<'a>> {
        self.params.clone()
    }
}

impl<'a> From<AliasLookup<'a>> for TokenizedCommand<'a> {
    fn from(lookup: AliasLookup<'a>) -> Self {
        lookup.lookup()
    }
}

impl<'a> From<TokenizedCommand<'a>> for Vec<AliasedToken<'a>> {
    fn from(value: TokenizedCommand<'a>) -> Self {
        let mut v = vec![];

        v.push(value.space);

        v.push(value.op);

        if let Some(flags) = value.flags {
            v.extend(flags);
        }
        if let Some(params) = value.params {
            v.push(params);
        }

        v
    }
}

// BUG this panics! with a SIGSEGV
// impl std::fmt::Display for AliasedToken<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         use quote::ToTokens;
//         compile_error!("this panics! with a SIGSEGV");
//
//         write!(
//             f,
//             "{}",
//             match self {
//                 Self::Space {
//                     space,
//                     alias,
//                     is_root,
//                 } =>
//                     if *is_root {
//                         "".into()
//                     } else {
//                         format!(
//                             "{}{}",
//                             space,
//                             if let Some(a) = alias {
//                                 format!("({})", a)
//                             } else {
//                                 "".into()
//                             }
//                         )
//                     },
//                 Self::Operation { op, alias } =>
//                     if self.is_direct() {
//                         "".into()
//                     } else {
//                         format!(
//                             "{}{}",
//                             op,
//                             if let Some(a) = alias {
//                                 format!("({})", a)
//                             } else {
//                                 "".into()
//                             }
//                         )
//                     },
//                 Self::Flag { flag, alias } => format!(
//                     "{}{}",
//                     flag,
//                     if let Some(a) = alias {
//                         format!("({})", a)
//                     } else {
//                         "".into()
//                     }
//                 ),
//                 Self::Params(ty) => format!("{}", ty.to_token_stream().to_string()),
//             }
//         )
//     }
// }

impl std::fmt::Display for TokenizedCommand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "S:{} O:{} F:{:?} P:{}",
            if !self.space.is_direct() {
                format!("{:?}", self.space)
            } else {
                "".into()
            },
            if !self.op.is_direct() {
                format!("{:?}", self.op)
            } else {
                "".into()
            },
            if let Some(flags) = &self.flags {
                flags
                    .into_iter()
                    .map(|f| format!("{:?}", f))
                    .fold(String::new(), |acc, s| acc + " " + &s)
            } else {
                "".into()
            },
            if let Some(params) = &self.params {
                format!("{:?}", params)
            } else {
                "".into()
            }
        )
    }
}
