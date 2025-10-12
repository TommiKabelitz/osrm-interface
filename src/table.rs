use thiserror::Error;

pub(crate) use crate::point::Point;

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct TableLocationEntry {
    pub hint: String,
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
}

pub struct TableRequestBuilder<'a> {
    pub sources: &'a [Point],
    pub destinations: &'a [Point],
    annotations: TableAnnotation,
    fallback_speed: Option<f64>,
    fallback_coordinate: Option<TableFallbackCoordinate>,
    scale_factor: Option<f64>,
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

        Ok(TableRequest {
            sources: self.sources,
            destinations: self.destinations,
            annotations: self.annotations,
            fallback_speed: self.fallback_speed,
            fallback_coordinate: self.fallback_coordinate,
            scale_factor: self.scale_factor,
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
