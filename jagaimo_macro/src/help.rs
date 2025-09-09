use std::collections::HashMap;

use syn::Ident;
use toml::Table;

use crate::output::type_tree::{OpType, RootType, SpaceType};
use resolve_calling_crate::ResolveCrate;

fn gen_delim(k: &mut String, d: usize) {
    let len = k.len();
    let delim = if len > d { len - d } else { 3 };
    k.push_str(&(0..delim).into_iter().map(|_| ' ').collect::<String>());
}

fn format_table(t: HashMap<String, String>, d: usize) -> String {
    t.into_iter()
        .map(|(mut k, v)| {
            gen_delim(&mut k, d);
            k + &v
        })
        .fold(String::new(), |acc, l| acc + &l + "\n")
}

pub trait Help
where
    Self: Sized,
{
    // formats the entire help message of self
    // and returns it
    fn help(self) -> String;

    // returns the cli tool version
    fn version() -> String {
        let [mut name, version] = ResolveCrate::new().read_manifest().crate_name_version();

        name.push_str(" ");
        name.push_str(&version);

        name
    }
}

pub fn read_help() -> toml::Table {
    std::fs::read_to_string(&ResolveCrate::new().into_string_suffixed("help.toml"))
        .unwrap()
        .parse::<Table>()
        .unwrap()
}

pub struct RootHelp<'a> {
    ident: &'a Ident,
    links: Option<HashMap<String, String>>,
    description: String,
    spaces: Option<HashMap<String, String>>,
    ops: Option<HashMap<String, String>>,
    flags: Option<HashMap<String, String>>,
    has_params: bool,
    has_direct_op: bool,
}

impl RootHelp<'_> {
    pub fn fmt(mut self) -> String {
        let desc = std::mem::take(&mut self.description);
        let links = self.links();
        let usage = self.usage();
        let flags = self.flags();
        let ops = self.ops();
        let spaces = self.spaces();

        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n",
            desc, links, usage, spaces, ops, flags,
        )
    }

    fn spaces(&mut self) -> String {
        if self.spaces.is_none() {
            return "".into();
        }
        let s = std::mem::take(&mut self.spaces).unwrap();

        format_table(s, 18)
    }

    fn ops(&mut self) -> String {
        if self.ops.is_none() {
            return "".into();
        }
        let o = std::mem::take(&mut self.ops).unwrap();

        format_table(o, 18)
    }

    fn links(&mut self) -> String {
        if self.links.is_none() {
            return "".into();
        }
        let l = std::mem::take(&mut self.links).unwrap();

        format_table(l, 6)
    }

    fn usage(&self) -> String {
        let i = self.ident;
        let o = if self.ops.is_none() {
            ""
        } else {
            " [OPERATION] "
        };
        let s = if self.spaces.is_none() {
            ""
        } else {
            " [NAMESPACE] "
        };
        let f = if self.flags.is_none() {
            ""
        } else {
            " [FLAGS] "
        };
        let p = if !self.has_params { "" } else { " [PARAMS] " };

        match [s.is_empty(), o.is_empty()] {
            [true, true] => format!("Usage: {} {}{}", i, p, f),
            [true, false] => {
                format!("Usage: {} {}{}\n", i, p, f) + &format!("Usage: {} {}{}{}", i, o, p, f)
            }
            [false, true] => {
                format!("Usage: {} {}{}\n", i, p, f) + &format!("Usage: {} {}{}{}", i, s, p, f)
            }
            [false, false] => {
                format!("Usage: {} {}{}\n", i, p, f)
                    + &format!("Usage: {} {}{}{}", i, o, p, f)
                    + &format!("Usage: {} {}{}{}{}", i, s, o, p, f)
            }
        }
    }

    fn flags(&mut self) -> String {
        if self.flags.is_none() {
            return "".into();
        }
        let f = std::mem::take(&mut self.flags).unwrap();

        format_table(f, 18)
    }
}

pub struct SpaceHelp<'a> {
    space: &'a Ident,
    links: Option<HashMap<String, String>>,
    description: String,
    ops: Option<HashMap<String, String>>,
    flags: Option<HashMap<String, String>>,
    has_params: bool,
    has_direct_op: bool,
}

impl SpaceHelp<'_> {
    pub fn fmt(mut self) -> String {
        let desc = std::mem::take(&mut self.description);
        let links = self.links();
        let usage = self.usage();
        let flags = self.flags();
        let ops = self.ops();

        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n",
            desc, links, usage, ops, flags,
        )
    }

    fn ops(&mut self) -> String {
        if self.ops.is_none() {
            return "".into();
        }
        let o = std::mem::take(&mut self.ops).unwrap();

        format_table(o, 18)
    }

    fn links(&mut self) -> String {
        if self.links.is_none() {
            return "".into();
        }
        let l = std::mem::take(&mut self.links).unwrap();

        format_table(l, 6)
    }

    fn usage(&self) -> String {
        let o = if self.ops.is_none() {
            ""
        } else {
            " [OPERATION] "
        };
        let s = self.space;
        let f = if self.flags.is_none() {
            ""
        } else {
            " [FLAGS] "
        };
        let p = if !self.has_params { "" } else { " [PARAMS] " };

        if !o.is_empty() {
            format!("Usage: {} {}{}\n", s, p, f) + &format!("Usage: {} {}{}{}", s, o, p, f)
        } else {
            format!("Usage: {} {}{}", s, p, f)
        }
    }

    fn flags(&mut self) -> String {
        if self.flags.is_none() {
            return "".into();
        }
        let f = std::mem::take(&mut self.flags).unwrap();

        format_table(f, 18)
    }
}

pub struct OpHelp<'a> {
    space: &'a Ident,
    op: &'a Ident,
    links: Option<HashMap<String, String>>,
    description: String,
    flags: Option<HashMap<String, String>>,
    has_params: bool,
}

impl OpHelp<'_> {
    pub fn fmt(mut self) -> String {
        let desc = std::mem::take(&mut self.description);
        let links = self.links();
        let usage = self.usage();
        let flags = self.flags();

        format!("{}\n\n{}\n\n{}\n\n{}\n\n", desc, links, usage, flags,)
    }

    fn links(&mut self) -> String {
        if self.links.is_none() {
            return "".into();
        }
        let l = std::mem::take(&mut self.links).unwrap();

        format_table(l, 6)
    }

    fn usage(&self) -> String {
        let o = self.op;
        let s = self.space;
        let f = if !self.flags.is_none() {
            ""
        } else {
            " [FLAGS] "
        };
        let p = if !self.has_params { "" } else { " [PARAMS] " };
        format!("{} {}{}{}", s, o, p, f)
    }

    fn flags(&mut self) -> String {
        if self.flags.is_none() {
            return "".into();
        }
        let f = std::mem::take(&mut self.flags).unwrap();

        format_table(f, 18)
    }
}

pub trait ExtractHelp<'a, T> {
    fn links(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn description(&self) -> String;

    fn spaces(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn ops(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn extract(self, has_params: bool, has_direct_op: bool) -> T;
}

// FIXME this needs to be in snake case
pub struct Extractor<'a> {
    toml: &'a toml::Table,
    space: Option<&'a Ident>,
    op: Option<&'a Ident>,
}

impl<'a> Extractor<'a> {
    pub fn new(space: Option<&'a Ident>, op: Option<&'a Ident>, toml: &'a toml::Table) -> Self {
        Self { toml, space, op }
    }
}

fn hashmap_from_table(table: Option<&Table>) -> Option<HashMap<String, String>> {
    if table.is_none() {
        return None;
    }

    Some(
        table
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.as_str().map(|v| v.to_owned())))
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k, v.unwrap()))
            .collect(),
    )
}

fn op_fallback_desc(space: &Ident, op: &Ident) -> String {
    format!("the {} operation from the {} namespace", op, space)
}

impl<'a> ExtractHelp<'a, OpHelp<'a>> for Extractor<'a> {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        let [space, op] = [
            self.space.map(|s| s.to_string()).unwrap(),
            self.op.map(|o| o.to_string()).unwrap(),
        ];

        let space = links.get(space);
        if let Some(tbl) = space {
            if let Some(op) = tbl.get(&op) {
                return hashmap_from_table(op.as_table());
            }
        } else {
            let fallback = links.get("_").map(|s| s.get(&op).map(|v| v.as_table()));
            if let Some(Some(t)) = fallback {
                return hashmap_from_table(t);
            }
        }

        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        let flags = self.toml.get("flags");
        if flags.is_none() {
            return None;
        }
        let flags = flags.unwrap();

        let [space, op] = [
            self.space.map(|s| s.to_string()).unwrap(),
            self.op.map(|o| o.to_string()).unwrap(),
        ];

        if let Some(ops) = flags.get(space) {
            if let Some(op) = ops.get(&op) {
                return hashmap_from_table(op.as_table());
            }
        } else {
            let fallback = flags.get("_").map(|s| s.get(&op).map(|v| v.as_table()));
            if let Some(Some(t)) = fallback {
                return hashmap_from_table(t);
            }
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descripts");
        if descs.is_none() {
            return op_fallback_desc(self.space.unwrap(), self.op.unwrap());
        }
        let descs = descs.unwrap();

        let [space, op] = [
            self.space.map(|s| s.to_string()).unwrap(),
            self.op.map(|o| o.to_string()).unwrap(),
        ];

        if let Some(ops) = descs.get(space) {
            if let Some(op) = ops.get(&op) {
                return op.as_str().map(|s| s.to_owned()).unwrap();
            }
        }

        descs
            .get("_")
            .map(|descs| {
                descs
                    .get(&op)
                    .map(|s| s.as_str().unwrap().to_owned())
                    .unwrap_or_else(|| op_fallback_desc(self.space.unwrap(), self.op.unwrap()))
            })
            .unwrap_or_else(|| op_fallback_desc(self.space.unwrap(), self.op.unwrap()))
    }

    fn extract(self, has_params: bool, has_direct_op: bool) -> OpHelp<'a> {
        let description = <Self as ExtractHelp<OpHelp>>::description(&self);
        let flags = <Self as ExtractHelp<OpHelp>>::flags(&self);
        let links = <Self as ExtractHelp<OpHelp>>::links(&self);

        OpHelp {
            space: self.space.unwrap(),
            op: self.op.unwrap(),
            has_params,
            description,
            links,
            flags,
        }
    }
}

fn space_fallback_desc(space: &Ident) -> String {
    format!("the {} namespace", space)
}

impl<'a> ExtractHelp<'a, SpaceHelp<'a>> for Extractor<'a> {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        let space = self.space.map(|s| s.to_string()).unwrap();

        if let Some(tbl) = links.get(space) {
            return hashmap_from_table(tbl.as_table());
        }

        None
    }

    fn ops(&self) -> Option<HashMap<String, String>> {
        let ops = self.toml.get("ops");
        if ops.is_none() {
            return None;
        }
        let ops = ops.unwrap();

        let space = self.space.map(|s| s.to_string()).unwrap();
        if let Some(ops) = ops.get(space) {
            return hashmap_from_table(ops.as_table());
        }

        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        let flags = self.toml.get("flags");
        if flags.is_none() {
            return None;
        }
        let flags = flags.unwrap();

        let space = self.space.map(|s| s.to_string()).unwrap();

        if let Some(flags) = flags.get(space) {
            return hashmap_from_table(flags.as_table());
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descripts");
        if descs.is_none() {
            return space_fallback_desc(self.space.unwrap());
        }
        let descs = descs.unwrap();

        let space = self.space.map(|s| s.to_string()).unwrap();

        if let Some(desc) = descs.get(space) {
            return desc.as_str().map(|s| s.to_owned()).unwrap();
        }

        space_fallback_desc(self.space.unwrap())
    }

    fn extract(self, has_params: bool, has_direct_op: bool) -> SpaceHelp<'a> {
        let description = <Self as ExtractHelp<SpaceHelp>>::description(&self);
        let flags = <Self as ExtractHelp<SpaceHelp>>::flags(&self);
        let links = <Self as ExtractHelp<SpaceHelp>>::links(&self);
        let ops = <Self as ExtractHelp<SpaceHelp>>::ops(&self);

        SpaceHelp {
            space: self.space.unwrap(),
            has_params,
            has_direct_op,
            ops,
            description,
            links,
            flags,
        }
    }
}

fn root_fallback_desc(space: &Ident) -> String {
    format!("the {} cli tool", space)
}

impl<'a> ExtractHelp<'a, RootHelp<'a>> for Extractor<'a> {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        if let Some(tbl) = links.get("root") {
            return hashmap_from_table(tbl.as_table());
        }

        None
    }

    fn ops(&self) -> Option<HashMap<String, String>> {
        let ops = self.toml.get("ops");
        if ops.is_none() {
            return None;
        }
        let ops = ops.unwrap();

        if let Some(ops) = ops.get("root") {
            return hashmap_from_table(ops.as_table());
        }

        None
    }

    fn spaces(&self) -> Option<HashMap<String, String>> {
        if let Some(spaces) = self.toml.get("spaces") {
            return hashmap_from_table(spaces.as_table());
        }

        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        let flags = self.toml.get("flags");
        if flags.is_none() {
            return None;
        }
        let flags = flags.unwrap();

        if let Some(flags) = flags.get("root") {
            return hashmap_from_table(flags.as_table());
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descripts");
        if descs.is_none() {
            return space_fallback_desc(self.space.unwrap());
        }
        let descs = descs.unwrap();

        if let Some(desc) = descs.get("root") {
            return desc.as_str().map(|s| s.to_owned()).unwrap();
        }

        root_fallback_desc(self.space.unwrap())
    }

    fn extract(self, has_params: bool, has_direct_op: bool) -> RootHelp<'a> {
        let description = <Self as ExtractHelp<RootHelp>>::description(&self);
        let flags = <Self as ExtractHelp<RootHelp>>::flags(&self);
        let links = <Self as ExtractHelp<RootHelp>>::links(&self);
        let ops = <Self as ExtractHelp<RootHelp>>::ops(&self);
        let spaces = <Self as ExtractHelp<RootHelp>>::spaces(&self);

        RootHelp {
            ident: self.space.unwrap(),
            has_params,
            has_direct_op,
            spaces,
            ops,
            description,
            links,
            flags,
        }
    }
}
