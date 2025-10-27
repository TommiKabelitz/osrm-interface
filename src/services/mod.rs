//! The services available to call.
//! Match, nearest, route, table and trip are available.
//!
//! There are no plans to support tile at this stage.

pub mod r#match;
pub mod nearest;
pub mod route;
pub mod table;
pub mod trip;

/// The array-like option for which there
/// was a dimension mismatch when constructing
/// a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimensionMismatch {
    Timestamps,
    Bearings,
    Radiuses,
    Hints,
    Approaches,
}

/// Allows restricting the direction on the road network at a waypoint.
/// Relative to the input coordinate.
///
/// This allows specification of being on the correct side of the road
/// ([`Approach::Curb`]) when arriving at a waypoint, being on the opposite
/// side, or being unrestricted.
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Approach {
    /// Require that the approach is made on the correct side of the road to
    /// the waypoint.
    Curb,
    /// Require that the approach is made on the opposite side of the road to
    /// the waypoint.
    Opposite,
    /// Make no requirements about how the waypoint is approached.
    Unrestricted,
}
impl Approach {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"curb"` or `"opposite"` or `"unrestricted`
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Curb => "curb",
            Self::Opposite => "opposite",
            Self::Unrestricted => "unrestricted",
        }
    }
}
