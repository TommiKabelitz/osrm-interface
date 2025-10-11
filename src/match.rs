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

pub struct MatchRequestBuilder<'a> {
    points: &'a [Point],
    steps: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
    annotations: bool,
    timestamps: Option<&'a [u64]>,
    radiuses: Option<&'a [f64]>,
    gaps: MatchGapsBehaviour,
    tidy: bool,
    waypoints: Option<&'a [usize]>,
}

impl<'a> MatchRequestBuilder<'a> {
    pub fn new(points: &'a [Point]) -> Self {
        Self {
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

    pub fn timestamps(mut self, val: &'a [u64]) -> Self {
        self.timestamps = Some(val);
        self
    }

    pub fn radiuses(mut self, val: &'a [f64]) -> Self {
        self.radiuses = Some(val);
        self
    }

    pub fn gaps(mut self, val: MatchGapsBehaviour) -> Self {
        self.gaps = val;
        self
    }

    pub fn tidy(mut self, val: bool) -> Self {
        self.tidy = val;
        self
    }

    pub fn waypoints(mut self, val: &'a [usize]) -> Self {
        self.waypoints = Some(val);
        self
    }

    pub fn build(self) -> Result<MatchRequest<'a>, MatchRequestError> {
        if self.points.len() < 2 {
            return Err(MatchRequestError::TooFewPoints);
        }

        if let Some(timestamps) = self.timestamps {
            if timestamps.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch);
            }
            if !timestamps.is_sorted() {
                return Err(MatchRequestError::TimestampsNotSorted);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.radiuses {
            if radiuses.len() != self.points.len() {
                return Err(MatchRequestError::DimensionMismatch);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(waypoints) = self.waypoints {
            if waypoints.is_empty() {
                return Err(MatchRequestError::EmptyWaypoints);
            }
            if let Some(max_idx) = waypoints.iter().max() {
                if *max_idx >= self.points.len() {
                    return Err(MatchRequestError::IndexOutOfBounds);
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
            radiuses: self.radiuses,
            gaps: self.gaps,
            tidy: self.tidy,
            waypoints: self.waypoints,
        })
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

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum MatchRequestError {
    TooFewPoints,
    DimensionMismatch,
    TimestampsNotSorted,
    EmptyWaypoints,
    IndexOutOfBounds,
}
