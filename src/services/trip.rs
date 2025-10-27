//! Given a set of coordinates, uses greedy heuristic to route the fastest
//! path through those coordinates.

use thiserror::Error;

use crate::{
    Point,
    osrm_response_types::{Route, TripWaypoint},
    request_types::{Bearing, Exclude, GeometryType, OverviewZoom, Snapping},
    services::{Approach, DimensionMismatch},
};

/// The request object passed to the trip service. Constructed
/// through [`TripRequestBuilder::build`] which verifies the
/// validity of the request.
///
/// See [`TripRequestBuilder`] for more information on trip requests.
#[derive(Clone, Debug)]
pub struct TripRequest<'a> {
    pub(crate) points: &'a [Point],
    pub(crate) roundtrip: bool,
    pub(crate) source: TripSource,
    pub(crate) destination: TripDestination,
    pub(crate) steps: bool,
    pub(crate) annotations: bool,
    pub(crate) geometry: GeometryType,
    pub(crate) overview: OverviewZoom,
    pub(crate) bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) radiuses: Option<&'a [Option<f64>]>,
    pub(crate) generate_hints: bool,
    pub(crate) hints: Option<&'a [Option<&'a str>]>,
    pub(crate) approaches: Option<&'a [Approach]>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
    pub(crate) skip_waypoints: bool,
}

/// Helper struct for building a [`TripRequest`].
///
/// Set options using the struct methods before calling
/// [`build`](Self::build). The `build` method validates the configuration
/// and attempts to detect invalid combinations before the request is sent
/// to the service.
///
/// ## Options
///
/// - **`points`** (*required*) — A slice of [`Point`]s that define the trip.  
///   Must contain at least two points.
///
/// - **`roundtrip`** (*default:* `false`) - Returned route returns to the start
///   location.
///
/// - **`source`** (*default:* `TripSource::Any`) - Which coordinate to start the
///   trip at.
///
/// - **`destination`** (*default:* `TripDestination::Any`) - Which coordinate to
///   end the trip at.
///
/// - **`steps`** (*default:* `false`) — If `true`, includes turn-by-turn navigation
///   instructions for each route leg.
///
/// - **`geometry`** (*default:* `GeometryType::Polyline`) — Specifies the encoding
///   format for returned route geometries. See [`GeometryType`] for options.
///
/// - **`overview`** (*default:* `OverviewZoom::Simplified`) — Controls the
///   generalization level of the route overview geometry. See [`OverviewZoom`].
///
/// - **`annotations`** (*default:* `false`) — When enabled, includes metadata such as
///   distance, duration, and speed for each segment of the route.
///
/// - **`exclude`** (*optional*) — A slice of [`Exclude`] values, all of the same
///   transport mode (e.g., all `Exclude::Car` or all `Exclude::Bicycle`),
///   specifying road classes to exclude from the route.
///
/// - **`snapping`** (*optional*) — Defines how input coordinates are snapped to
///   the road network. See [`Snapping`] for available modes.
///
/// - **`skip_waypoints`** (*default:* `false`) — When `true`, OSRM omits waypoint
///   information from the response, reducing payload size.
///
/// - **`generate_hints`** (*default:* `true`) — When enabled, OSRM returns
///   location hints to accelerate subsequent queries.
///
/// ## Array options
///
/// The following options require array slices as input.
/// Each array must have the same length as `points`.
/// Some options use [`Option`] to allow per-point overrides.
///
/// - **`bearings`** (*optional*) — A slice of optional [`Bearing`]s, one per point.
///   Each defines an allowed direction in which the point may be snapped to a node.
///   None => Any direction.
///
/// - **`radiuses`** (*optional*) — A slice of optional radiuses (in meters),
///   constraining how far OSRM may search from each coordinate. None => infinite.
///
/// - **`hints`** (*optional*) — A slice of optional pre-computed location hints,
///   one per point, to accelerate lookups for known coordinates. Unspecified hints
///   will be snapped.
///
/// - **`approaches`** (*optional*) — A slice of [`Approach`] values specifying
///   the side of the road (e.g., `Approach::Curb`, `Approach::Unrestricted`)
///   to approach each waypoint from. `Approach::Unrestricted` is the default behaviour.
///
/// ## Example
///
/// ```
/// use osrm_interface::{
///     Point,
///     request_types::OverviewZoom,
///     trip::{TripRequestBuilder, TripSource},
/// };
/// let points = [
///     Point::new(48.040437, 10.316550).expect("Invalid point"),
///     Point::new(49.006101, 9.052887).expect("Invalid point"),
///     Point::new(48.942296, 10.510960).expect("Invalid point"),
/// ];
/// let trip_request = TripRequestBuilder::new(&points)
///     .roundtrip(true)
///     .source(TripSource::First)
///     .overview(OverviewZoom::Full)
///     .annotations(true)
///     .build()
///     .expect("Failed to build TripRequest");
/// ```
#[derive(Clone, Debug)]
pub struct TripRequestBuilder<'a> {
    points: &'a [Point],
    steps: bool,
    annotations: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
    roundtrip: bool,
    source: TripSource,
    destination: TripDestination,
    bearings: Option<&'a [Option<Bearing>]>,
    radiuses: Option<&'a [Option<f64>]>,
    generate_hints: bool,
    hints: Option<&'a [Option<&'a str>]>,
    approaches: Option<&'a [Approach]>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
    skip_waypoints: bool,
}

impl<'a> TripRequestBuilder<'a> {
    /// Creates a new [`TripRequestBuilder`] with default parameters.
    ///
    /// The builder can then be customized using its setter methods.
    pub fn new(points: &'a [Point]) -> Self {
        Self {
            points,
            steps: false,
            annotations: false,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::False,
            roundtrip: true,
            source: TripSource::Any,
            destination: TripDestination::Any,
            bearings: None,
            radiuses: None,
            generate_hints: true,
            hints: None,
            approaches: None,
            exclude: None,
            snapping: None,
            skip_waypoints: false,
        }
    }

    /// Overwrite the points provided at construction of the builder. Useful
    /// for reusing a builder with the same options.
    ///
    /// Take care that array-like options are still the same length as points,
    /// [`build`](Self::build) will throw an error if not.
    pub fn points(mut self, points: &'a [Point]) -> Self {
        self.points = points;
        self
    }

    /// Sets whether to include turn-by-turn navigation steps in the response.
    pub fn steps(mut self, steps: bool) -> Self {
        self.steps = steps;
        self
    }

    /// Sets whether to include per-segment annotations in the route.
    pub fn annotations(mut self, annotations: bool) -> Self {
        self.annotations = annotations;
        self
    }

    /// Sets the geometry encoding type for the route response.
    pub fn geometry(mut self, geometry: GeometryType) -> Self {
        self.geometry = geometry;
        self
    }

    /// Sets the overview simplification level for the route.
    pub fn overview(mut self, overview: OverviewZoom) -> Self {
        self.overview = overview;
        self
    }

    /// Whether the trip should return to the starting point.
    pub fn roundtrip(mut self, roundtrip: bool) -> Self {
        self.roundtrip = roundtrip;
        self
    }

    /// Whether the trip must start at the first point or anywhere.
    pub fn source(mut self, source: TripSource) -> Self {
        self.source = source;
        self
    }

    /// Whether the trip must end at the last point or anywhere.
    pub fn destination(mut self, destination: TripDestination) -> Self {
        self.destination = destination;
        self
    }

    /// Sets per-point bearings to constrain the direction snapping to
    /// the node.
    ///
    /// Each bearing must correspond to the point at the same index. Passing
    /// None allows snapping in any direction.
    pub fn bearings(mut self, bearings: &'a [Option<Bearing>]) -> Self {
        self.bearings = Some(bearings);
        self
    }

    /// Sets per-point search radiuses (in meters) for coordinate snapping.
    ///
    /// Each radius must correspond to the point at the same index. Radii must
    /// be positive. Passing None corresponds to an infinite search radius.
    pub fn radiuses(mut self, coordinate_radiuses: &'a [Option<f64>]) -> Self {
        self.radiuses = Some(coordinate_radiuses);
        self
    }

    /// Sets per-point search radiuses (in meters) for coordinate snapping.
    ///
    /// Each radius must correspond to the point at the same index. Radii must
    /// be positive. Passing None corresponds to an infinite search radius.
    pub fn generate_hints(mut self, generate_hints: bool) -> Self {
        self.generate_hints = generate_hints;
        self
    }

    /// Sets precomputed location hints for faster coordinate matching.
    ///
    /// Each hint corresponds to the point at the same index. OSRM will
    /// use the hint rather than the point information where supplied.
    ///
    /// Passing hints will result in radiuses, bearings,
    /// approaches being ignored for that point.
    pub fn hints(mut self, coordinate_hints: &'a [Option<&'a str>]) -> Self {
        self.hints = Some(coordinate_hints);
        self
    }

    /// Sets per-point approaches to control the side of the road to access from.
    pub fn approaches(mut self, approach_direction: &'a [Approach]) -> Self {
        self.approaches = Some(approach_direction);
        self
    }

    /// Sets which road classes should be excluded from route generation.
    ///
    /// All excludes must belong to the same transport mode.
    pub fn exclude(mut self, exclude: &'a [Exclude]) -> Self {
        self.exclude = Some(exclude);
        self
    }

    /// Sets the snapping behavior for input coordinates.
    pub fn snapping(mut self, snapping: Snapping) -> Self {
        self.snapping = Some(snapping);
        self
    }

    /// Sets whether to skip including waypoint data in the response.
    pub fn skip_waypoints(mut self, skip_waypoints: bool) -> Self {
        self.skip_waypoints = skip_waypoints;
        self
    }

    /// Builds a [`TripRequest`] from the configured parameters.
    ///
    /// Performs validation to ensure all per-point array options
    /// have consistent lengths and compatible types.
    ///
    /// # Errors
    ///
    /// Returns a [`TripRequestError`] if:
    /// - Fewer than two points were provided.
    /// - Array lengths do not match the number of points.
    /// - Exclude types mix transport modes.
    /// - Any radii are negative.
    pub fn build(&self) -> Result<TripRequest<'a>, TripRequestError> {
        if self.points.len() < 2 {
            return Err(TripRequestError::InsufficientPoints);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.bearings {
            if bearings.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.radiuses {
            if radiuses.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
            if !radiuses.iter().all(|r| r.is_none_or(|v| v >= 0.0)) {
                return Err(TripRequestError::NegativeRadius);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.hints {
            if hints.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.approaches {
            if approaches.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Approaches,
                ));
            }
        }
        #[allow(clippy::collapsible_if)]
        if let Some(exclude) = self.exclude {
            if !exclude.is_empty() {
                if !match exclude[0] {
                    Exclude::Car(_) => exclude.iter().all(|e| matches!(e, Exclude::Car(_))),
                    Exclude::Bicycle(_) => exclude.iter().all(|e| matches!(e, Exclude::Bicycle(_))),
                } {
                    return Err(TripRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(TripRequest {
            points: self.points,
            steps: self.steps,
            annotations: self.annotations,
            geometry: self.geometry,
            overview: self.overview,
            roundtrip: self.roundtrip,
            source: self.source,
            destination: self.destination,
            bearings: self.bearings,
            radiuses: self.radiuses,
            generate_hints: self.generate_hints,
            hints: self.hints,
            approaches: self.approaches,
            exclude: self.exclude,
            snapping: self.snapping,
            skip_waypoints: self.skip_waypoints,
        })
    }
}

/// The comprehensive error type returned when attempting to
/// construct an invalid [`TripRequest`].
#[derive(Error, Debug)]
pub enum TripRequestError {
    /// Trip requires at least 2 points.
    #[error("Trip requires at least 2 points")]
    InsufficientPoints,
    /// Mismatch of number of elements between points and one
    /// of the array-like options.
    #[error("Mismatch of dimensions between Points and {0:?}")]
    DimensionMismatch(DimensionMismatch),
    /// Cannot mix excludes of different [`Exclude`] variants.
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
    /// Radius values must be non-negative.
    #[error("Radii must be non-negative")]
    NegativeRadius,
}

/// The response type returned by the Trip service.
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TripResponse {
    /// The response code returned by the service. `"Ok"` denotes
    /// success, `"NoTrips"` suggests input coordinates are not
    /// connected.
    pub code: String,
    /// The [`Route`] objects that assemble the trace.
    pub trips: Vec<Route>,
    /// The waypoints in the trip, in **input** order. Only `None`
    /// when `skip_waypoints` is set to `true`.
    pub waypoints: Option<Vec<TripWaypoint>>,
}

/// For specifying whether a trip may start anywhere or only
/// at the first provided point.
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum TripSource {
    Any,
    First,
}
impl TripSource {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"any"` or `"first"`
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::First => "first",
        }
    }
}

/// For specifying whether a trip may end anywhere or only
/// at the last provided point.
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum TripDestination {
    Any,
    Last,
}
impl TripDestination {
    /// Formats the variant as a lowercase &str. The form expected
    /// by `osrm-routed`.
    ///
    /// eg. `"any"` or `"last"`
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Last => "last",
        }
    }
}
