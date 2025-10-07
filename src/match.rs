use crate::{
    osrm_response_types::{MatchRoute, MatchWaypoint},
    point::Point,
    request_types::{GeometryType, OverviewZoom},
};

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MatchRequest<'a> {
    pub points: &'a [Point],
    pub steps: bool,
    pub geometry: GeometryType,
    pub overview: OverviewZoom,
    pub annotations: bool,
    pub timestamps: Option<&'a [u64]>,
    pub radiuses: Option<&'a [f64]>,
    pub gaps: MatchGapsBehaviour,
    pub tidy: bool,
    pub waypoints: Option<&'a [usize]>,
}
impl<'a> MatchRequest<'a> {
    pub fn new(points: &'a [Point]) -> Option<Self> {
        if points.len() < 2 {
            return None;
        }
        Some(Self {
            points,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::Simplified,
            steps: false,
            annotations: false,
            timestamps: None,
            radiuses: None,
            gaps: MatchGapsBehaviour::Split,
            tidy: false,
            waypoints: None,
        })
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

    pub fn with_timestamps(
        mut self,
        timestamps: &'a [u64],
    ) -> Result<Self, (Self, MatchRequestError)> {
        if timestamps.len() != self.points.len() {
            return Err((self, MatchRequestError::DimensionMismatch));
        }
        if !timestamps.is_sorted() {
            return Err((self, MatchRequestError::TimestampsNotSorted));
        }
        self.timestamps = Some(timestamps);
        Ok(self)
    }

    pub fn with_radiuses(mut self, radiuses: &'a [f64]) -> Result<Self, (Self, MatchRequestError)> {
        if radiuses.len() != self.points.len() {
            return Err((self, MatchRequestError::DimensionMismatch));
        }
        self.radiuses = Some(radiuses);
        Ok(self)
    }

    pub fn with_gaps(mut self, gaps_behaviour: MatchGapsBehaviour) -> Self {
        self.gaps = gaps_behaviour;
        self
    }

    pub fn with_tidy(mut self) -> Self {
        self.tidy = false;
        self
    }

    pub fn with_waypoints(
        mut self,
        waypoints: &'a [usize],
    ) -> Result<Self, (Self, MatchRequestError)> {
        if waypoints.is_empty() {
            return Err((self, MatchRequestError::EmptyWaypoints));
        }
        if *waypoints.iter().max().unwrap() > self.points.len() - 1 {
            return Err((self, MatchRequestError::IndexMismatch));
        }
        self.waypoints = Some(waypoints);
        Ok(self)
    }
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

pub enum MatchRequestError {
    DimensionMismatch,
    IndexMismatch,
    EmptyWaypoints,
    TimestampsNotSorted,
}
