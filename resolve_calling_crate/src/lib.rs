#![cfg_attr(feature = "nightly", feature(doc_auto_cfg))]
#![cfg_attr(feature = "nightly", feature(test))]
pub mod manifest;
pub mod resolve_crate;

pub use manifest::ReadManifest;
pub use resolve_crate::ResolveCrate;
