//! Given a set of source and destination coordinates, determine the
//! distance and/or duration to travel between those locations.

use thiserror::Error;

use crate::Point;
use crate::{
    r#match::{Approach, DimensionMismatch},
    request_types::{Bearing, Exclude, Snapping},
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct TableLocationEntry {
    /// Hint is only null when generate_hints=false
    pub hint: Option<String>,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TableRequest<'a> {
    pub(crate) sources: &'a [Point],
    pub(crate) destinations: &'a [Point],
    pub(crate) annotations: TableAnnotation,
    pub(crate) fallback_speed: Option<f64>,
    pub(crate) fallback_coordinate: Option<TableFallbackCoordinate>,
    pub(crate) scale_factor: Option<f64>,
    pub(crate) source_bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) destination_bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) source_radiuses: Option<&'a [Option<f64>]>,
    pub(crate) destination_radiuses: Option<&'a [Option<f64>]>,
    pub(crate) generate_hints: bool,
    pub(crate) source_hints: Option<&'a [Option<&'a str>]>,
    pub(crate) destination_hints: Option<&'a [Option<&'a str>]>,
    pub(crate) source_approaches: Option<&'a [Approach]>,
    pub(crate) destination_approaches: Option<&'a [Approach]>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
}

pub struct TableRequestBuilder<'a> {
    pub sources: &'a [Point],
    pub destinations: &'a [Point],
    annotations: TableAnnotation,
    fallback_speed: Option<f64>,
    fallback_coordinate: Option<TableFallbackCoordinate>,
    scale_factor: Option<f64>,
    source_bearings: Option<&'a [Option<Bearing>]>,
    destination_bearings: Option<&'a [Option<Bearing>]>,
    source_radiuses: Option<&'a [Option<f64>]>,
    destination_radiuses: Option<&'a [Option<f64>]>,
    generate_hints: bool,
    source_hints: Option<&'a [Option<&'a str>]>,
    destination_hints: Option<&'a [Option<&'a str>]>,
    source_approaches: Option<&'a [Approach]>,
    destination_approaches: Option<&'a [Approach]>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
}

impl<'a> TableRequestBuilder<'a> {
    pub fn new(sources: &'a [Point], destinations: &'a [Point]) -> Self {
        Self {
            sources,
            destinations,
            annotations: TableAnnotation::Duration,
            fallback_speed: None,
            fallback_coordinate: None,
            scale_factor: None,
            source_bearings: None,
            destination_bearings: None,
            source_radiuses: None,
            destination_radiuses: None,
            generate_hints: true,
            source_hints: None,
            destination_hints: None,
            source_approaches: None,
            destination_approaches: None,
            exclude: None,
            snapping: None,
        }
    }

    pub fn annotations(mut self, val: TableAnnotation) -> Self {
        self.annotations = val;
        self
    }

    pub fn fallback(mut self, coord: TableFallbackCoordinate, speed: f64) -> Self {
        self.fallback_coordinate = Some(coord);
        self.fallback_speed = Some(speed);
        self
    }

    pub fn scale_factor(mut self, val: f64) -> Self {
        self.scale_factor = Some(val);
        self
    }

    pub fn source_bearings(mut self, source_bearings: &'a [Option<Bearing>]) -> Self {
        self.source_bearings = Some(source_bearings);
        self
    }

    pub fn destination_bearings(mut self, destination_bearings: &'a [Option<Bearing>]) -> Self {
        self.destination_bearings = Some(destination_bearings);
        self
    }

    pub fn source_radiuses(mut self, source_coordinate_radiuses: &'a [Option<f64>]) -> Self {
        self.source_radiuses = Some(source_coordinate_radiuses);
        self
    }

    pub fn destination_radiuses(
        mut self,
        destination_coordinate_radiuses: &'a [Option<f64>],
    ) -> Self {
        self.destination_radiuses = Some(destination_coordinate_radiuses);
        self
    }

    pub fn generate_hints(mut self, generate_hints: bool) -> Self {
        self.generate_hints = generate_hints;
        self
    }

    pub fn source_hints(mut self, source_coordinate_hints: &'a [Option<&'a str>]) -> Self {
        self.source_hints = Some(source_coordinate_hints);
        self
    }

    pub fn destination_hints(
        mut self,
        destination_coordinate_hints: &'a [Option<&'a str>],
    ) -> Self {
        self.destination_hints = Some(destination_coordinate_hints);
        self
    }

    pub fn source_approaches(mut self, source_approach_direction: &'a [Approach]) -> Self {
        self.source_approaches = Some(source_approach_direction);
        self
    }

    pub fn destination_approaches(
        mut self,
        destination_approach_direction: &'a [Approach],
    ) -> Self {
        self.destination_approaches = Some(destination_approach_direction);
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

    pub fn build(self) -> Result<TableRequest<'a>, TableRequestError> {
        if self.sources.is_empty() {
            return Err(TableRequestError::EmptySources);
        }
        if self.destinations.is_empty() {
            return Err(TableRequestError::EmptyDestinations);
        }
        #[allow(clippy::collapsible_if)]
        if let Some(s) = self.fallback_speed {
            if s <= 0.0 {
                return Err(TableRequestError::NonPositiveFallbackSpeed);
            }
        }
        #[allow(clippy::collapsible_if)]
        if let Some(f) = self.scale_factor {
            if f <= 0.0 {
                return Err(TableRequestError::NonPositiveScaleFactor);
            }
        }

        if self.fallback_coordinate.is_some() ^ self.fallback_speed.is_some() {
            return Err(TableRequestError::IncompleteFallbackPair);
        }
        if self.scale_factor.is_some()
            && matches!(
                self.annotations,
                TableAnnotation::None | TableAnnotation::Distance
            )
        {
            return Err(TableRequestError::ScaleFactorRequiresDuration);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.source_bearings {
            if bearings.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.destination_bearings {
            if bearings.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.source_radiuses {
            if radiuses.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.destination_radiuses {
            if radiuses.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.source_hints {
            if hints.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.destination_hints {
            if hints.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.source_approaches {
            if approaches.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Approaches,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.destination_approaches {
            if approaches.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
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
                    return Err(TableRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(TableRequest {
            sources: self.sources,
            destinations: self.destinations,
            annotations: self.annotations,
            fallback_speed: self.fallback_speed,
            fallback_coordinate: self.fallback_coordinate,
            scale_factor: self.scale_factor,
            source_bearings: self.source_bearings,
            destination_bearings: self.destination_bearings,
            source_radiuses: self.source_radiuses,
            destination_radiuses: self.destination_radiuses,
            generate_hints: self.generate_hints,
            source_hints: self.source_hints,
            destination_hints: self.destination_hints,
            source_approaches: self.source_approaches,
            destination_approaches: self.destination_approaches,
            exclude: self.exclude,
            snapping: self.snapping,
        })
    }
}

#[derive(Error, Debug)]
pub enum TableRequestError {
    #[error("No sources in table request")]
    EmptySources,
    #[error("No destinations in table request")]
    EmptyDestinations,
    #[error("Fallback speed must be greater than 0.0")]
    NonPositiveFallbackSpeed,
    #[error("Scale factor must be greater than 0.0")]
    NonPositiveScaleFactor,
    #[error(
        "Fallback speed and coordinate are co-dependent and must both be specified, or both null"
    )]
    IncompleteFallbackPair,
    #[error(
        "Scale factor multiplies duration, so annotations must be TableAnnotation::Duration, or TableAnnotation::All"
    )]
    ScaleFactorRequiresDuration,
    #[error("Mismatch of dimensions between Points and {0:?}")]
    DimensionMismatch(DimensionMismatch),
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum TableAnnotation {
    None = 0,
    Duration = 1,
    Distance = 2,
    All = 3,
}
impl TableAnnotation {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Duration => "duration",
            Self::Distance => "distance",
            Self::All => "duration,distance",
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum TableFallbackCoordinate {
    Input = 0,
    Snapped = 1,
}
impl TableFallbackCoordinate {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Snapped => "snapped",
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct TableResponse {
    /// If the request was successful "Ok" otherwise see the service dependent and general status codes
    pub code: String,
    pub destinations: Vec<TableLocationEntry>,
    /// Array of arrays that stores the matrix in row-major order. `durations[i][j]` gives the travel time from the i-th source to the j-th destination. Values are given in seconds. Can be `null` if no route between `i` and `j` can be found
    pub durations: Option<Vec<Vec<Option<f64>>>>,
    /// Array of arrays that stores the matrix in row-major order. `distances[i][j]` gives the travel distance from the i-th source to the j-th destination. Values are given in meters. Can be `null` if no route between `i` and `j` can be found
    pub distances: Option<Vec<Vec<Option<f64>>>>,
    pub sources: Vec<TableLocationEntry>,
}
