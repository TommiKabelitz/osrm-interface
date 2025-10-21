use thiserror::Error;

use crate::r#match::{Approach, DimensionMismatch};
use crate::osrm_response_types::{Route, Waypoint};
use crate::request_types::{Bearing, Exclude, OverviewZoom, Snapping};
use crate::{point::Point, request_types::GeometryType};

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

pub struct RouteRequestBuilder<'a> {
    pub points: &'a [Point],
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

    pub fn alternatives(mut self, val: bool) -> Self {
        self.alternatives = val;
        self
    }

    pub fn steps(mut self, val: bool) -> Self {
        self.steps = val;
        self
    }

    pub fn annotations(mut self, val: bool) -> Self {
        self.annotations = val;
        self
    }

    pub fn geometry(mut self, val: GeometryType) -> Self {
        self.geometry = val;
        self
    }

    pub fn overview(mut self, val: OverviewZoom) -> Self {
        self.overview = val;
        self
    }

    pub fn continue_straight(mut self, val: bool) -> Self {
        self.continue_straight = val;
        self
    }
    pub fn bearings(mut self, bearings: &'a [Option<Bearing>]) -> Self {
        self.bearings = Some(bearings);
        self
    }

    pub fn radiuses(mut self, coordinate_radiuses: &'a [Option<f64>]) -> Self {
        self.radiuses = Some(coordinate_radiuses);
        self
    }

    pub fn generate_hints(mut self, generate_hints: bool) -> Self {
        self.generate_hints = generate_hints;
        self
    }

    pub fn hints(mut self, coordinate_hints: &'a [Option<&'a str>]) -> Self {
        self.hints = Some(coordinate_hints);
        self
    }

    pub fn approaches(mut self, approach_direction: &'a [Approach]) -> Self {
        self.approaches = Some(approach_direction);
        self
    }
    pub fn exclude(mut self, exclude: &'a [Exclude]) -> Self {
        self.exclude = Some(exclude);
        self
    }

    pub fn snapping(mut self, snapping: Snapping) -> Self {
        self.snapping = Some(snapping);
        self
    }

    pub fn skip_waypoints(mut self, skip_waypoints: bool) -> Self {
        self.skip_waypoints = skip_waypoints;
        self
    }

    pub fn build(self) -> Result<RouteRequest<'a>, RouteRequestError> {
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

#[derive(Error, Debug)]
pub enum RouteRequestError {
    #[error("Route requires at least 2 points")]
    InsufficientPoints,
    #[error("Mismatch of dimensions between Points and {0:?}")]
    DimensionMismatch(DimensionMismatch),
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct RouteResponse {
    /// If the request was successful "Ok" otherwise see the service dependent and general status codes
    pub code: String,
    /// An array of `Route` objects, ordered by descending recommendation rank
    pub routes: Vec<Route>,
    /// Array of `Waypoint` objects representing all waypoints in order. Only None
    /// when the request is passed
    pub waypoints: Option<Vec<Waypoint>>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct SimpleRouteResponse {
    pub code: String,
    pub durations: f64,
    pub distance: f64,
}
