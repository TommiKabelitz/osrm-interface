//! Common request sub-types that are used to build the service requests

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum GeometryType {
    Polyline = 0,
    Polyline6 = 1,
    GeoJSON = 2,
}
impl GeometryType {
    pub fn url_form(self) -> &'static str {
        match self {
            Self::GeoJSON => "geojson",
            Self::Polyline => "polyline",
            Self::Polyline6 => "polyline6",
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum OverviewZoom {
    Simplified = 0,
    Full = 1,
    False = 2,
}
impl OverviewZoom {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Simplified => "simplified",
            Self::False => "false",
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Profile {
    Car,
    Bike,
    Foot,
}
impl Profile {
    pub fn url_form(self) -> &'static str {
        match self {
            Self::Bike => "bike",
            Self::Car => "car",
            Self::Foot => "foot",
        }
    }
}

/// A direction in which OSRM may seek a segment to snap to, relative
/// to the given coordinate. Holds two fields:
///
/// - `bearing` is the midpoint of the arc in degrees, clockwise
///   from true north.
/// - `range` the number of degrees in each direction. Hence,
///   the full size of the arc is `2 * range`.
///
/// So to seek a segment in an arc 10 degrees either side of North, use
/// `Bearing::new(0,10)`. For an arc 90 degrees either side of east, so
/// from north to south, use `Bearing::new(90,90)`.
///
/// `bearing` must be in the range `[0,360]` and `range` in `[0,180]`, both
/// inclusive.
///
/// Constructing with [`new`](Self::new) will check `bearing` and `range` values.
/// [`new`](Self::new_unchecked) is also provided.
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bearing {
    bearing: i16,
    range: i16,
}

impl Bearing {
    /// Check 0 <= bearing <= 360 and 0 <= range <= 180.
    ///
    /// Returns `None` if that is not satisfied. Also see
    /// [`new_unchecked`](Self::new_unchecked).
    pub fn new(bearing: i16, range: i16) -> Option<Self> {
        if !(0..=360).contains(&bearing) & !(0..=180).contains(&range) {
            return None;
        }
        Some(Self { bearing, range })
    }

    /// Init without checking bearing and range.
    ///
    /// OSRM will reject invalid `Bearings` when a request is made
    /// to a service.
    pub fn new_unchecked(bearing: i16, range: i16) -> Self {
        Self { bearing, range }
    }

    /// Formats the bearing as a comma separated pair. The form expected
    /// by `osrm-routed`.
    ///
    /// `format!("{},{}", self.bearing, self.range)`.
    pub fn url_form(&self) -> String {
        format!("{},{}", self.bearing, self.range)
    }
}

impl Default for Bearing {
    fn default() -> Self {
        Bearing {
            bearing: 0,
            range: 180,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum Exclude {
    Car(CarExclude),
    Bicycle(BicycleExclude),
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum CarExclude {
    Toll,
    Motorway,
    Ferry,
}

impl CarExclude {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Toll => "toll",
            Self::Motorway => "motorway",
            Self::Ferry => "ferry",
        }
    }
}

/// The default Bike profile does not enable
/// exclusion of ferry, so this is not guaranteed
/// to work as expected, but it does exist in the code.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum BicycleExclude {
    Ferry,
}

impl BicycleExclude {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ferry => "ferry",
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum Snapping {
    Default = 0,
    Any = 1,
}
impl Snapping {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Any => "any",
        }
    }
}
