//! The remote version of the OSRM engine which calls into the Web API of
//! OSRM. Locked behind the `remote` feature flag.

mod osrm_engine;
#[cfg_attr(doc, doc(cfg(feature = "remote")))]
pub use osrm_engine::OsrmEngine;
