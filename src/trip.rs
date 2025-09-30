use itertools::Itertools;

use crate::{
    point::Point,
    request_types::{GeometryType, OverviewZoom},
    route::Route,
    waypoints::Waypoint,
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TripRequest<'a> {
    pub points: &'a [Point],
    pub steps: bool,
    pub annotations: bool,
    pub geometry: GeometryType,
    pub overview: OverviewZoom,
}

impl<'a> TripRequest<'a> {
    pub fn new(points: &'a [Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        Some(Self {
            points,
            steps: false,
            annotations: false,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::Simplified,
        })
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
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct TripResponse {
    pub code: String,
    pub trips: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}
impl TripResponse {
    pub fn full_geometry(&self) -> String {
        self.trips.iter().map(|r| r.geometry.clone()).join("")
    }
}
