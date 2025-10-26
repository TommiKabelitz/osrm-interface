//! Given a set of coordinates (and optionally timestamps) determine
//! the likely route taken. Matching those coordinates to a route.

use thiserror::Error;

use crate::{
    Point,
    osrm_response_types::{MatchRoute, MatchWaypoint},
    request_types::{Bearing, Exclude, GeometryType, OverviewZoom, Snapping},
};

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MatchRequest<'a> {
    pub(crate) points: &'a [Point],
    pub(crate) steps: bool,
    pub(crate) geometry: GeometryType,
    pub(crate) overview: OverviewZoom,
    pub(crate) annotations: bool,
    pub(crate) timestamps: Option<&'a [u64]>,
    pub(crate) gaps: MatchGapsBehaviour,
    pub(crate) tidy: bool,
    pub(crate) waypoints: Option<&'a [usize]>,
    pub(crate) bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) radiuses: Option<&'a [Option<f64>]>,
    pub(crate) generate_hints: bool,
    pub(crate) hints: Option<&'a [Option<&'a str>]>,
    pub(crate) approaches: Option<&'a [Approach]>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
    pub(crate) skip_waypoints: bool,
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

/// Helper struct for building a [`MatchRequest`].
///
/// Set options using the struct methods before calling
/// [`build`](Self::build). The `build` method attempts to
/// check for invalid requests before the request is passed
/// to the service
///
/// ## Options:
///
/// - **`points`** (*required*) — A slice of [`Point`]s to match along the road network.
///   Must contain at least two points.
///
/// - **`steps`** (*default:* `false`) — If `true`, includes turn-by-turn navigation
///   instructions for each matched segment.
///
/// - **`geometry`** (*default:* `GeometryType::Polyline`) — Specifies the encoding
///   format for the returned geometry. See [`GeometryType`] for options.
///
/// - **`overview`** (*default:* `OverviewZoom::Simplified`) — Controls the
///   generalization level of the route overview geometry. See [`OverviewZoom`].
///
/// - **`annotations`** (*default:* `false`) — When enabled, includes metadata such as
///   distance, duration, and speed values for each segment.
///
/// - **`gaps`** (*default:* `MatchGapsBehaviour::Split`) — Defines how to handle gaps
///   in GPS traces. Options are:
///   - `Split`: Splits the trace into sub-traces at gaps, requires `timestamps`.
///   - `Ignore`: Attempts to match through gaps without splitting.
///
/// - **`tidy`** (*default:* `false`) — If `true`, removes outlier points before matching.
///
/// - **`waypoints`** (*optional*) — A slice of indices (into `points`) marking which
///   points should be treated as waypoints. Must not be empty or out of bounds.
///
/// - **`exclude`** (*optional*) — A slice of [`Exclude`] values, all of the same
///   transport mode (e.g., only `Exclude::Car` or only `Exclude::Bicycle`),
///   specifying road classes to exclude.
///
/// - **`snapping`** (*optional*) — Defines how input coordinates are snapped to
///   the road network. See [`Snapping`] for available modes.
///
/// - **`skip_waypoints`** (*default:* `false`) — When `true`, OSRM omits waypoint
///   information from the response, reducing payload size. Useful if only
///   distance and/or duration are required.
///
/// - **`generate_hints`** (*default:* `true`) — When enabled, OSRM will return
///   location hints that can speed up subsequent queries.
///
/// ## Array options
///
/// The following options require array slices as input. Each input maps 1-1 with the
/// corresponding [`Point`] by index. Some options allow optional array values which
/// allow behaviour to be specified only for particular points.
///
/// - **`timestamps`** (*optional*) — A slice of UNIX timestamps (in seconds) matching
///   the `points` array length. Must be sorted in ascending order. Required when
///   `gaps` is set to [`MatchGapsBehaviour::Split`].
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
/// use osrm_interface::r#match::{MatchRequestBuilder, MatchGapsBehaviour};
///
/// let points = [
///     Point::new(48.040437, 10.316550).expect("Invalid point"),
///     Point::new(49.006101, 9.052887).expect("Invalid point"),
///     Point::new(48.942296, 10.510960).expect("Invalid point"),
///     Point::new(51.248931, 7.594814).expect("Invalid point"),
/// ];
///
/// let match_request = MatchRequestBuilder::new(&points)
///     .generate_hints(true)
///     .gaps(MatchGapsBehaviour::Ignore)
///     .annotations(true)
///     .build()
///     .expect("Failed to build MatchRequest");
/// ```
pub struct MatchRequestBuilder<'a> {
    points: &'a [Point],
    steps: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
    annotations: bool,
    timestamps: Option<&'a [u64]>,
    gaps: MatchGapsBehaviour,
    tidy: bool,
    waypoints: Option<&'a [usize]>,
    bearings: Option<&'a [Option<Bearing>]>,
    radiuses: Option<&'a [Option<f64>]>,
    generate_hints: bool,
    hints: Option<&'a [Option<&'a str>]>,
    approaches: Option<&'a [Approach]>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
    skip_waypoints: bool,
}

impl<'a> MatchRequestBuilder<'a> {
    /// Creates a new [`MatchRequestBuilder`] with the required list of `points`.
    ///
    /// Default values are applied to all other options.
    pub fn new(points: &'a [Point]) -> Self {
        Self {
            points,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::Simplified,
            steps: false,
            annotations: false,
            timestamps: None,
            gaps: MatchGapsBehaviour::Split,
            tidy: false,
            waypoints: None,
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

    /// Includes or omits step-by-step navigation instructions in the response.
    pub fn steps(mut self, include_steps: bool) -> Self {
        self.steps = include_steps;
        self
    }

    /// Includes or omits metadata (distance, duration, etc.) for each segment.
    pub fn annotations(mut self, include_annotations: bool) -> Self {
        self.annotations = include_annotations;
        self
    }

    /// Sets the geometry encoding format used in the response.
    pub fn geometry(mut self, geometry_type: GeometryType) -> Self {
        self.geometry = geometry_type;
        self
    }

    /// Sets the overview simplification level for the returned route geometry.
    pub fn overview(mut self, overview_zoom: OverviewZoom) -> Self {
        self.overview = overview_zoom;
        self
    }

    /// Sets the timestamps (in seconds) corresponding to each input point.
    ///
    /// Must be sorted in ascending order and have the same length as `points`.
    /// Required when using [`MatchGapsBehaviour::Split`].
    pub fn timestamps(mut self, timestamps: &'a [u64]) -> Self {
        self.timestamps = Some(timestamps);
        self
    }

    /// Sets how to handle gaps in the input trace (split or ignore).
    ///
    /// `MatchGapsBehaviour::Split` requires timestamps are set.
    pub fn gaps(mut self, gaps_behaviour: MatchGapsBehaviour) -> Self {
        self.gaps = gaps_behaviour;
        self
    }

    /// Enables or disables tidying of input points (removes outliers).
    pub fn tidy(mut self, do_tidy: bool) -> Self {
        self.tidy = do_tidy;
        self
    }

    /// Specifies which input indices should be treated as waypoints.
    ///
    /// Must not be empty or contain out-of-bounds indices.
    pub fn waypoints(mut self, waypoint_indices: &'a [usize]) -> Self {
        self.waypoints = Some(waypoint_indices);
        self
    }

    /// Specifies the allowed snapping direction for each point.
    ///
    /// Must have the same length as `points`.
    /// Null values will allow snapping in all directions.
    pub fn bearings(mut self, bearings: &'a [Option<Bearing>]) -> Self {
        self.bearings = Some(bearings);
        self
    }

    /// Sets the search radius (in meters) for each coordinate.
    ///
    /// Must have the same length as `points`.
    /// Default radius is infinite.
    pub fn radiuses(mut self, coordinate_radiuses: &'a [Option<f64>]) -> Self {
        self.radiuses = Some(coordinate_radiuses);
        self
    }

    /// Enables or disables automatic hint generation.
    ///
    /// Hints can speed up subsequent queries for similar coordinates.
    pub fn generate_hints(mut self, generate_hints: bool) -> Self {
        self.generate_hints = generate_hints;
        self
    }

    /// Set pre-generated hints for each coordinate.
    ///
    /// Must have the same length as `points`.
    /// Unspecified coordinates will be snapped.
    pub fn hints(mut self, coordinate_hints: &'a [Option<&'a str>]) -> Self {
        self.hints = Some(coordinate_hints);
        self
    }

    /// Specifies the approach direction for each coordinate.
    ///
    /// Must have the same length as `points`.
    /// Default is `Approach::Unrestricted`.
    pub fn approaches(mut self, approach_direction: &'a [Approach]) -> Self {
        self.approaches = Some(approach_direction);
        self
    }

    /// Excludes specific road classes or transport modes from routing.
    ///
    /// All entries must be of the same variant (e.g., all `Exclude::Car`).
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

    /// Validates and constructs the [`MatchRequest`].
    ///
    /// Returns an error if configuration is invalid — for example:
    /// - Too few points ([`MatchRequestError::TooFewPoints`])
    /// - Mismatched array lengths ([`MatchRequestError::DimensionMismatch`])
    /// - Missing timestamps when `gaps` = `Split` ([`MatchRequestError::TimestampsRequiredForSplitBehaviour`])
    /// - Timestamps not sorted ([`MatchRequestError::TimestampsNotSorted`])
    /// - Out-of-bounds waypoint indices ([`MatchRequestError::WaypointIndexOutOfBounds`])
    /// - Mixed `Exclude` types ([`MatchRequestError::DifferentExcludeTypes`])
    pub fn build(self) -> Result<MatchRequest<'a>, MatchRequestError> {
        if self.points.len() < 2 {
            return Err(MatchRequestError::TooFewPoints);
        }

        if let Some(timestamps) = self.timestamps {
            if timestamps.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch(
                    DimensionMismatch::Timestamps,
                ));
            }
            if !timestamps.is_sorted() {
                return Err(MatchRequestError::TimestampsNotSorted);
            }
        } else if let MatchGapsBehaviour::Split = self.gaps {
            return Err(MatchRequestError::TimestampsRequiredForSplitBehaviour);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(waypoints) = self.waypoints {
            if waypoints.is_empty() {
                return Err(MatchRequestError::EmptyWaypoints);
            }
            if let Some(max_idx) = waypoints.iter().max() {
                if *max_idx >= self.points.len() {
                    return Err(MatchRequestError::WaypointIndexOutOfBounds(
                        *max_idx,
                        self.points.len(),
                    ));
                }
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.bearings {
            if bearings.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.radiuses {
            if radiuses.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
            if !radiuses.iter().all(|r| r.is_none_or(|v| v >= 0.0)) {
                return Err(MatchRequestError::NegativeRadius);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.hints {
            if hints.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.approaches {
            if approaches.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch(
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
                    return Err(MatchRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(MatchRequest {
            points: self.points,
            steps: self.steps,
            geometry: self.geometry,
            overview: self.overview,
            annotations: self.annotations,
            timestamps: self.timestamps,
            gaps: self.gaps,
            tidy: self.tidy,
            waypoints: self.waypoints,
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

#[derive(Error, Debug)]
pub enum MatchRequestError {
    #[error("Match requires at least 2 points")]
    TooFewPoints,
    #[error("Mismatch of dimensions between Points and {0:?}")]
    DimensionMismatch(DimensionMismatch),
    #[error("Timestamps must be increasing order")]
    TimestampsNotSorted,
    #[error("Timestamps must be included for GapsBehaviour::Split")]
    TimestampsRequiredForSplitBehaviour,
    #[error("Waypoints when non-None must have non-zero length")]
    EmptyWaypoints,
    #[error("Waypoints contain index {0} which is out of bounds for points with size {1}")]
    WaypointIndexOutOfBounds(usize, usize),
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
    #[error("Radii must be non-negative")]
    NegativeRadius,
}

#[derive(Debug)]
pub enum DimensionMismatch {
    Timestamps,
    Bearings,
    Radiuses,
    Hints,
    Approaches,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum MatchGapsBehaviour {
    Split = 0,
    Ignore = 1,
}
impl MatchGapsBehaviour {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Split => "split",
            Self::Ignore => "ignore",
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct MatchResponse {
    pub code: String,
    pub tracepoints: Vec<Option<MatchWaypoint>>,
    pub matchings: Vec<MatchRoute>,
}
