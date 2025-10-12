use thiserror::Error;

use crate::{
    osrm_response_types::{Route, Waypoint},
    point::Point,
    request_types::{GeometryType, OverviewZoom},
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TripRequest<'a> {
    pub(crate) points: &'a [Point],
    pub(crate) steps: bool,
    pub(crate) annotations: bool,
    pub(crate) geometry: GeometryType,
    pub(crate) overview: OverviewZoom,
}

pub struct TripRequestBuilder<'a> {
    pub points: &'a [Point],
    steps: bool,
    annotations: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
}

impl<'a> TripRequestBuilder<'a> {
    pub fn new(points: &'a [Point]) -> Self {
        Self {
            points,
            steps: false,
            annotations: false,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::False,
        }
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

    pub fn build(self) -> Result<TripRequest<'a>, TripRequestError> {
        if self.points.len() < 2 {
            return Err(TripRequestError::InsufficientPoints);
        }
        Ok(TripRequest {
            points: self.points,
            steps: self.steps,
            annotations: self.annotations,
            geometry: self.geometry,
            overview: self.overview,
        })
    }
}

#[derive(Error, Debug)]
pub enum TripRequestError {
    #[error("Trip requires at least 2 points")]
    InsufficientPoints,
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
