//! Common request sub-types that are used to build the service requests

/// Specify which geometry type the service should return.
///
/// For no geometry, set `OverviewZoom::False` in the builder.
///
/// See [`Geometry`](crate::osrm_response_types::Geometry) for more
/// context about the output format.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum GeometryType {
    Polyline = 0,
    /// Polyline format, but with 6 decimal points of precision rather
    /// than 5.
    Polyline6 = 1,
    GeoJSON = 2,
}
impl GeometryType {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"geojson"` or `"polyline"` or `"polyline6`
    pub fn url_form(self) -> &'static str {
        match self {
            Self::GeoJSON => "geojson",
            Self::Polyline => "polyline",
            Self::Polyline6 => "polyline6",
        }
    }
}

/// Specify the level of detail for the
/// [`Geometry`](crate::osrm_response_types::Geometry) returned by
/// the service.
///
/// Setting this as `False` will result in the geometry field being
/// `None` in the response.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.#[derive(Clone, Copy)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum OverviewZoom {
    Simplified = 0,
    Full = 1,
    False = 2,
}
impl OverviewZoom {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"full"` or `"simplified"` or `"false`
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Simplified => "simplified",
            Self::False => "false",
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

/// A namespace for the different classes of excludes. Bicycle and
/// Car excludes cannot be mixed as they are dependent on the map
/// profile.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum Exclude {
    Car(CarExclude),
    Bicycle(BicycleExclude),
}

/// Types of nodes from which Car routing may exclude.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum CarExclude {
    Toll,
    Motorway,
    Ferry,
}
impl CarExclude {
    /// Formats the variant as a lowercase &str.
    ///
    /// eg. `"toll"`, `"motorway"`, `"ferry"`
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Toll => "toll",
            Self::Motorway => "motorway",
            Self::Ferry => "ferry",
        }
    }
}

/// Types of nodes from which Bike routing may exclude.
///
/// The default Bike profile does not enable
/// exclusion of ferry, so this is not guaranteed
/// to work as expected, but it does exist in the code.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
pub enum BicycleExclude {
    Ferry,
}

impl BicycleExclude {
    /// Formats the variant as a lowercase &str.
    ///
    /// eg. `"ferry"`
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ferry => "ferry",
        }
    }
}

/// Specify which nodes may be used for snapping.
///
/// With [`Snapping::Default`], input coordinates are snapped to
/// _accessible_ road segments. This excludes segments marked as
/// `is_startpoint = false` in the profile. This includes private
/// driveways or links intended for exit routing.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum Snapping {
    /// Only snap input coordinates to _accesible_ road segments.
    Default = 0,
    /// Snap the input coordinate to any node.
    Any = 1,
}
impl Snapping {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"any"` or `"default"`
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Any => "any",
        }
    }
}
