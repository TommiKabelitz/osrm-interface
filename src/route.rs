use thiserror::Error;

use crate::osrm_response_types::{Route, Waypoint};
use crate::request_types::{Exclude, OverviewZoom};
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
    pub(crate) exclude: Option<&'a [Exclude]>,
}

pub struct RouteRequestBuilder<'a> {
    pub points: &'a [Point],
    alternatives: bool,
    steps: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
    annotations: bool,
    continue_straight: bool,
    exclude: Option<&'a [Exclude]>,
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
            exclude: None,
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

    pub fn exclude(mut self, exclude: &'a [Exclude]) -> Self {
        self.exclude = Some(exclude);
        self
    }

    pub fn build(self) -> Result<RouteRequest<'a>, RouteRequestError> {
        if self.points.len() < 2 {
            return Err(RouteRequestError::InsufficientPoints);
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
            exclude: self.exclude,
        })
    }
}

#[derive(Error, Debug)]
pub enum RouteRequestError {
    #[error("Route requires at least 2 points")]
    InsufficientPoints,
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
    /// Array of `Waypoint` objects representing all waypoints in order
    pub waypoints: Vec<Waypoint>,
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
