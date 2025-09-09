use std::env;
use std::fs;

use crate::ReadManifest;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ResolveCrate {
    who: String,
}

impl ResolveCrate {
    /// returns a new instance of Self
    ///
    /// the actual value returned may be wrong
    /// if the last else block was reached
    ///
    /// but that should be handled while still returning Self
    /// instead of retunring an Option or a Result
    /// because these is no reason for this to encounter a failure
    /// when cargo is already compiling the crate
    ///
    /// tldr; this must return Self with a path for which crate is running/compiling
    pub fn new() -> Self {
        let who = if let Ok(who) = env::var("CARGO_MANIFEST_DIR") {
            who + "/"
        } else if let Ok(who) = env::var("CARGO_PKG_NAME") {
            let cwd = env::current_dir().unwrap();
            cwd.into_os_string()
                .into_string()
                .expect("if this was not a proper string, the earlier var calls would have failed")
                + "/"
                + &who
                + "/"
        } else {
            let m = fs::read_to_string("Cargo.toml").unwrap();
            if let (true, Ok(p)) = (m.contains("[workspace]"), env::var("CARGO_CRATE_NAME")) {
                p + "/"
            } else {
                "./".into()
            }
        };

        Self { who }
    }

    /// wrapper call around From<&Self> impl for ReadManifest
    ///
    /// returns a ReadManifest instance
    #[cfg(feature = "manifest")]
    pub fn read_manifest(&self) -> ReadManifest {
        self.into()
    }

    /// wrapper call around From<Self> impl for ReadManifest
    ///
    /// returns a ReadManifest instance
    ///
    /// consumes self
    #[cfg(feature = "manifest")]
    pub fn into_manifest(self) -> ReadManifest {
        self.into()
    }

    /// returns &str of the inner value of the running crate path
    pub fn as_str(&self) -> &str {
        &self.who
    }

    /// clones and returns an owned String of the inner value of the running crate path
    pub fn to_string(&self) -> String {
        self.who.clone()
    }

    /// consumes self and returns an owned String of the inner value of the running crate path
    pub fn into_string(self) -> String {
        self.who
    }

    /// takes self and anything that can be referenced as an str
    ///
    /// returns an owned String
    ///
    /// consumes self, for a non-consuming version, see `to_string_suffixed`
    pub fn into_string_suffixed<T: AsRef<str>>(self, suffix: T) -> String {
        self.who + suffix.as_ref()
    }

    /// takes self and anything that can be referenced as an str
    ///
    /// returns an owned String with the path of the compiling crate suffixed with the given
    /// argument string value
    pub fn to_string_suffixed(&self, suffix: impl AsRef<str>) -> String {
        self.who.clone() + suffix.as_ref()
    }

    /// prefixes the passed argument's string value with the compiling crate path
    pub fn prefix_str(&self, path: impl AsRef<str>) -> String {
        self.as_str().chars().chain(path.as_ref().chars()).collect()
    }
}

impl AsRef<str> for ResolveCrate {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for ResolveCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "crate foudn at <{}>", self.who)
    }
}
