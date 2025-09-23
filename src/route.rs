use crate::request_types::OverviewZoom;
use crate::waypoints::Waypoint;
use crate::{point::Point, request_types::GeometryType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
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
        if points.is_empty() {
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
    pub fn with_alternatives(mut self, val: bool) -> Self {
        self.alternatives = val;
        self
    }

    pub fn with_steps(mut self, val: bool) -> Self {
        self.steps = val;
        self
    }

    pub fn with_annotations(mut self, val: bool) -> Self {
        self.annotations = val;
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

    pub fn with_continue_straight(mut self, val: bool) -> Self {
        self.continue_straight = val;
        self
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SimpleRouteResponse {
    pub code: String,
    pub durations: f64,
    pub distance: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RouteResponse {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsrmResponse {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Route {
    pub legs: Vec<Leg>,
    pub weight_name: String,
    pub geometry: String,
    pub weight: f64,
    pub duration: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Leg {
    pub steps: Vec<Step>,
    pub weight: f64,
    pub summary: String,
    pub duration: f64,
    pub distance: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Step {}
