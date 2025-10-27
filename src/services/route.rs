//! Given a set of coordinates, construct a route through those coordinates
//! in the supplied order.

use thiserror::Error;

use crate::osrm_response_types::{Route, Waypoint};
use crate::request_types::{Bearing, Exclude, OverviewZoom, Snapping};
use crate::services::{Approach, DimensionMismatch};
use crate::{Point, request_types::GeometryType};

/// The request object passed to the route service. Constructed
/// through [`RouteRequestBuilder::build`] which verifies the
/// validity of the request.
///
/// See [`RouteRequestBuilder`] for more information on route requests.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RouteRequest<'a> {
    pub(crate) points: &'a [Point],
    pub(crate) alternatives: bool,
    pub(crate) steps: bool,
    pub(crate) geometry: GeometryType,
    pub(crate) overview: OverviewZoom,
    pub(crate) annotations: bool,
    pub(crate) continue_straight: bool,
    pub(crate) bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) radiuses: Option<&'a [Option<f64>]>,
    pub(crate) generate_hints: bool,
    pub(crate) hints: Option<&'a [Option<&'a str>]>,
    pub(crate) approaches: Option<&'a [Approach]>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
    pub(crate) skip_waypoints: bool,
}

/// Helper struct for building a [`RouteRequest`].
///
/// Set route options using the struct methods before calling
/// [`build`](Self::build). The `build` method validates the configuration
/// and attempts to detect invalid combinations before the request is sent
/// to the service.
///
/// ## Options
///
/// - **`points`** (*required*) — A slice of [`Point`]s that define the route path.  
///   Must contain at least two points.
///
/// - **`alternatives`** (*default:* `false`) — If `true`, OSRM will return
///   alternative routes in addition to the recommended one.
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
/// - **`continue_straight`** (*default:* `true`) — If `true`, the route will continue
///   straight at waypoints where possible. If `false`, U-turns may be allowed
///   when starting new route legs.
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
/// use osrm_interface::{Point, request_types::OverviewZoom, route::RouteRequestBuilder};
/// let points = [
///     Point::new(48.040437, 10.316550).expect("Invalid point"),
///     Point::new(49.006101, 9.052887).expect("Invalid point"),
///     Point::new(48.942296, 10.510960).expect("Invalid point"),
/// ];
/// let route_request = RouteRequestBuilder::new(&points)
///     .steps(true)
///     .alternatives(true)
///     .overview(OverviewZoom::Full)
///     .annotations(true)
///     .build()
///     .expect("Failed to build RouteRequest");
/// ```
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RouteRequestBuilder<'a> {
    points: &'a [Point],
    alternatives: bool,
    steps: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
    annotations: bool,
    continue_straight: bool,
    bearings: Option<&'a [Option<Bearing>]>,
    radiuses: Option<&'a [Option<f64>]>,
    generate_hints: bool,
    hints: Option<&'a [Option<&'a str>]>,
    approaches: Option<&'a [Approach]>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
    skip_waypoints: bool,
}

impl<'a> RouteRequestBuilder<'a> {
    /// Creates a new [`RouteRequestBuilder`] with default parameters.
    ///
    /// The builder can then be customized using its setter methods.
    pub fn new(points: &'a [Point]) -> Self {
        Self {
            points,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::Simplified,
            alternatives: false,
            steps: false,
            annotations: false,
            continue_straight: true,
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

    /// Sets whether to request alternative routes.
    pub fn alternatives(mut self, generate_alternatives: bool) -> Self {
        self.alternatives = generate_alternatives;
        self
    }

    /// Sets whether to include turn-by-turn navigation steps in the response.
    pub fn steps(mut self, generate_steps: bool) -> Self {
        self.steps = generate_steps;
        self
    }

    /// Sets whether to include per-segment annotations in the route.
    pub fn annotations(mut self, include_annotations: bool) -> Self {
        self.annotations = include_annotations;
        self
    }

    /// Sets the geometry encoding type for the route response.
    pub fn geometry(mut self, geometry_type: GeometryType) -> Self {
        self.geometry = geometry_type;
        self
    }

    /// Sets the overview simplification level for the route.
    pub fn overview(mut self, overview_detail: OverviewZoom) -> Self {
        self.overview = overview_detail;
        self
    }

    /// Sets whether the route should continue straight at waypoints where possible.
    pub fn continue_straight(mut self, prefer_continue_straight: bool) -> Self {
        self.continue_straight = prefer_continue_straight;
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

    /// Builds a [`RouteRequest`] from the configured parameters.
    ///
    /// Performs validation to ensure all per-point array options
    /// have consistent lengths and compatible types.
    ///
    /// # Errors
    ///
    /// Returns a [`RouteRequestError`] if:
    /// - Fewer than two points were provided.
    /// - Array lengths do not match the number of points.
    /// - Exclude types mix transport modes.
    pub fn build(&self) -> Result<RouteRequest<'a>, RouteRequestError> {
        if self.points.len() < 2 {
            return Err(RouteRequestError::InsufficientPoints);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.bearings {
            if bearings.len() != self.points.len() {
                return Err(RouteRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.radiuses {
            if radiuses.len() != self.points.len() {
                return Err(RouteRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
            if !radiuses.iter().all(|r| r.is_none_or(|v| v >= 0.0)) {
                return Err(RouteRequestError::NegativeRadius);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.hints {
            if hints.len() != self.points.len() {
                return Err(RouteRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.approaches {
            if approaches.len() != self.points.len() {
                return Err(RouteRequestError::DimensionMismatch(
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
                    return Err(RouteRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(RouteRequest {
            points: self.points,
            alternatives: self.alternatives,
            steps: self.steps,
            geometry: self.geometry,
            overview: self.overview,
            annotations: self.annotations,
            continue_straight: self.continue_straight,
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
/// construct an invalid [`RouteRequest`].
#[derive(Error, Debug)]
pub enum RouteRequestError {
    // Route requires at least 2 points.
    #[error("Route requires at least 2 points")]
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

/// The response type returned by the Route service.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
///
/// Implements [`serde::Deserialize`] if either of `feature="native"`
/// or `feature="remote"` are set.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct RouteResponse {
    /// The response code returned by the service. `"Ok"` denotes
    /// success, `"NoRoute"` suggests input coordinates are not
    /// connected.
    pub code: String,
    /// An array of `Route` objects, ordered by descending recommendation rank
    pub routes: Vec<Route>,
    /// Array of `Waypoint` objects representing all waypoints in order. Only `None`
    /// when `skip_waypoints` is set to `true`.
    pub waypoints: Option<Vec<Waypoint>>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SimpleRouteResponse {
    pub code: String,
    pub durations: f64,
    pub distance: f64,
}
