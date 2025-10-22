use thiserror::Error;

use crate::{
    r#match::{Approach, DimensionMismatch},
    osrm_response_types::{Route, Waypoint},
    point::Point,
    request_types::{Bearing, Exclude, GeometryType, OverviewZoom, Snapping},
};

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TripRequest<'a> {
    pub(crate) points: &'a [Point],
    pub(crate) steps: bool,
    pub(crate) annotations: bool,
    pub(crate) geometry: GeometryType,
    pub(crate) overview: OverviewZoom,
    pub(crate) roundtrip: bool,
    pub(crate) source: TripSource,
    pub(crate) destination: TripDestination,
    pub(crate) bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) radiuses: Option<&'a [Option<f64>]>,
    pub(crate) generate_hints: bool,
    pub(crate) hints: Option<&'a [Option<&'a str>]>,
    pub(crate) approaches: Option<&'a [Approach]>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
    pub(crate) skip_waypoints: bool,
}

pub struct TripRequestBuilder<'a> {
    pub points: &'a [Point],
    steps: bool,
    annotations: bool,
    geometry: GeometryType,
    overview: OverviewZoom,
    roundtrip: bool,
    source: TripSource,
    destination: TripDestination,
    bearings: Option<&'a [Option<Bearing>]>,
    radiuses: Option<&'a [Option<f64>]>,
    generate_hints: bool,
    hints: Option<&'a [Option<&'a str>]>,
    approaches: Option<&'a [Approach]>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
    skip_waypoints: bool,
}

impl<'a> TripRequestBuilder<'a> {
    pub fn new(points: &'a [Point]) -> Self {
        Self {
            points,
            steps: false,
            annotations: false,
            geometry: GeometryType::Polyline,
            overview: OverviewZoom::False,
            roundtrip: true,
            source: TripSource::Any,
            destination: TripDestination::Any,
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

    pub fn steps(mut self, steps: bool) -> Self {
        self.steps = steps;
        self
    }

    pub fn annotations(mut self, annotations: bool) -> Self {
        self.annotations = annotations;
        self
    }

    pub fn geometry(mut self, geometry: GeometryType) -> Self {
        self.geometry = geometry;
        self
    }

    pub fn overview(mut self, overview: OverviewZoom) -> Self {
        self.overview = overview;
        self
    }

    pub fn roundtrip(mut self, roundtrip: bool) -> Self {
        self.roundtrip = roundtrip;
        self
    }

    pub fn source(mut self, source: TripSource) -> Self {
        self.source = source;
        self
    }

    pub fn destination(mut self, destination: TripDestination) -> Self {
        self.destination = destination;
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

    pub fn build(self) -> Result<TripRequest<'a>, TripRequestError> {
        if self.points.len() < 2 {
            return Err(TripRequestError::InsufficientPoints);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.bearings {
            if bearings.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.radiuses {
            if radiuses.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
        }
        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.hints {
            if hints.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.approaches {
            if approaches.len() != self.points.len() {
                return Err(TripRequestError::DimensionMismatch(
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
                    return Err(TripRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(TripRequest {
            points: self.points,
            steps: self.steps,
            annotations: self.annotations,
            geometry: self.geometry,
            overview: self.overview,
            roundtrip: self.roundtrip,
            source: self.source,
            destination: self.destination,
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
pub enum TripRequestError {
    #[error("Trip requires at least 2 points")]
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
pub struct TripResponse {
    pub code: String,
    pub trips: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
#[repr(C)]
pub enum TripSource {
    Any,
    First,
}
impl TripSource {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::First => "first",
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, Copy)]
#[repr(C)]
pub enum TripDestination {
    Any,
    Last,
}
impl TripDestination {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Last => "last",
        }
    }
}
