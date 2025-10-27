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
#[derive(Debug)]
pub enum DimensionMismatch {
    Timestamps,
    Bearings,
    Radiuses,
    Hints,
    Approaches,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
#[repr(C)]
pub enum Approach {
    Curb,
    Opposite,
    Unrestricted,
}
impl Approach {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Curb => "curb",
            Self::Opposite => "opposite",
            Self::Unrestricted => "unrestricted",
        }
    }
}
