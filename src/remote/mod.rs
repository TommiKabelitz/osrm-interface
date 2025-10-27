//! The remote version of the OSRM engine which calls into the Web API of
//! OSRM. Locked behind the `remote` feature flag.
//!
//! ## Profiles
//!
//! When making a request, the full URL which contains the request is
//! constructed. This URL is of form
//!
//! ```ignore
//! GET /{service}/{version}/{profile}/{coordinates}[.{format}]?option=value&option=value
//! ```
//!
//! In this form, profile may be selected. When making a request directly
//! to the backend where `osrm-routed` is running, profile is ignored as it
//! is defined by the profile used when the map was extracted.
//!
//! Profile is present so that when making requests to the ProjectOSRM endpoint,
//! it can dispatch to the correct routed instance.

mod osrm_engine;
#[cfg_attr(doc, doc(cfg(feature = "remote")))]
pub use osrm_engine::OsrmEngine;

/// The profile with which the underlying map data was extracted.
///
/// The profile is placed in the URL. In many cases, it is ignored.
/// See [`crate::remote`] for more information about `Profile`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Car,
    Bike,
    Foot,
}
impl Profile {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"bike"` or `"car"` or `"foot"`
    pub fn url_form(self) -> &'static str {
        match self {
            Self::Bike => "bike",
            Self::Car => "car",
            Self::Foot => "foot",
        }
    }
}
