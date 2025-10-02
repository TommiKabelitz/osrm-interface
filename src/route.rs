use crate::osrm_response_types::{Route, Waypoint};
use crate::request_types::OverviewZoom;
use crate::{point::Point, request_types::GeometryType};

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RouteRequest<'a> {
    pub points: &'a [Point],
    pub alternatives: bool,
    pub steps: bool,
    pub geometry: GeometryType,
    pub overview: OverviewZoom,
    pub annotations: bool,
    pub continue_straight: bool,
}
impl<'a> RouteRequest<'a> {
    pub fn new(points: &'a [Point]) -> Option<Self> {
        if points.len() < 2 {
            return None;
        }
        Some(Self {
            points,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::Simplified,
            alternatives: false,
            steps: false,
            annotations: false,
            continue_straight: true,
        })
    }
    pub fn with_alternatives(mut self) -> Self {
        self.alternatives = true;
        self
    }

    pub fn with_steps(mut self) -> Self {
        self.steps = true;
        self
    }

    pub fn with_annotations(mut self) -> Self {
        self.annotations = true;
        self
    }

    pub fn with_geometry(mut self, val: GeometryType) -> Self {
        self.geometry = val;
        self
    }

    pub fn with_overview(mut self, val: OverviewZoom) -> Self {
        self.overview = val;
        self
    }

    pub fn with_continue_straight(mut self) -> Self {
        self.continue_straight = true;
        self
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct RouteResponse {
    pub code: String,
    pub routes: Vec<Route>,
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
