#![cfg(feature = "manifest")]
use crate::ResolveCrate;

/// struct for reading the Cargo.toml manifest file of a crate
///
/// has a few useful methods such as `crate_name` and `crate_version`
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ReadManifest {
    manifest: String,
}

impl From<&ResolveCrate> for ReadManifest {
    fn from(resolver: &ResolveCrate) -> Self {
        Self {
            manifest: std::fs::read_to_string(resolver.to_string_suffixed("Cargo.toml")).unwrap(),
        }
    }
}

impl From<ResolveCrate> for ReadManifest {
    fn from(resolver: ResolveCrate) -> Self {
        Self {
            manifest: std::fs::read_to_string(resolver.into_string_suffixed("Cargo.toml")).unwrap(),
        }
    }
}

impl ReadManifest {
    /// returns an `[String; 2]` of the manifest package name and package version
    /// respectively
    ///
    /// refer to methods `crate_name` and `crate_version` for failure cases
    pub fn crate_name_version(&self) -> [String; 2] {
        let mut iter = self
            .manifest
            .lines()
            .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
            .map(|l| l.to_owned());

        // NOTE could do collect -> try_into
        // but "name =" may exist for every  [[bin]] [[test]] or [[example]]
        // and there is no guarentee that there would be no second version key somewhere
        [iter.next().unwrap(), iter.next().unwrap()]
    }

    /// return a `&str` of the package table's name key's value
    /// as found in the Cargo.toml manifest file
    ///
    /// if some other name key in a different table from [package]
    /// preceeds the package.name key,
    /// then this would return the wrong value
    ///
    /// if no name key exists in the manifest, then
    /// this would return an empty &str ""
    pub fn crate_name(&self) -> &str {
        self.manifest
            .lines()
            .find(|l| l.starts_with("name = "))
            .map(|l| &l[8..l.len() - 1])
            .unwrap_or("")
    }

    /// return a `&str` of the package table's version key's value
    /// as found in the Cargo.toml manifest file
    ///
    /// if some other version key in a different table from [package]
    /// preceeds the package.version key,
    /// then this would return the wrong value
    ///
    /// if no version key exists in the manifest, then
    /// this would return an empty &str ""
    pub fn crate_version(&self) -> &str {
        self.manifest
            .lines()
            .find(|l| l.starts_with("version = "))
            .map(|l| &l[10..l.len() - 1])
            .unwrap_or("")
    }

    /// returns the &str value of the manifest file data
    pub fn as_str(&self) -> &str {
        &self.manifest
    }

    /// clones and returns the String value of the manifest file data
    pub fn to_string(&self) -> String {
        self.manifest.clone()
    }

    /// consumes self and returns the String value of the nmanifest file data
    pub fn into_string(self) -> String {
        self.manifest
    }
}

impl From<ReadManifest> for String {
    fn from(value: ReadManifest) -> String {
        value.manifest
    }
}

impl std::fmt::Display for ReadManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.manifest)
    }
}
