use thiserror::Error;

use crate::{
    osrm_response_types::{MatchRoute, MatchWaypoint},
    point::Point,
    request_types::{Bearing, GeometryType, OverviewZoom},
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
}

#[cfg_attr(feature = "debug", derive(Debug))]
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

pub struct MatchRequestBuilder<'a> {
    pub points: &'a [Point],
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
            gaps: MatchGapsBehaviour::Split,
            tidy: false,
            waypoints: None,
            bearings: None,
            radiuses: None,
            generate_hints: true,
            hints: None,
            approaches: None,
        }
    }

    pub fn steps(mut self, include_steps: bool) -> Self {
        self.steps = include_steps;
        self
    }

    pub fn annotations(mut self, include_annotations: bool) -> Self {
        self.annotations = include_annotations;
        self
    }

    pub fn geometry(mut self, geometry_type: GeometryType) -> Self {
        self.geometry = geometry_type;
        self
    }

    pub fn overview(mut self, overview_zoom: OverviewZoom) -> Self {
        self.overview = overview_zoom;
        self
    }

    pub fn timestamps(mut self, timestamps: &'a [u64]) -> Self {
        self.timestamps = Some(timestamps);
        self
    }

    pub fn gaps(mut self, gaps_behaviour: MatchGapsBehaviour) -> Self {
        self.gaps = gaps_behaviour;
        self
    }

    pub fn tidy(mut self, do_tidy: bool) -> Self {
        self.tidy = do_tidy;
        self
    }

    pub fn waypoints(mut self, waypoint_indices: &'a [usize]) -> Self {
        self.waypoints = Some(waypoint_indices);
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
