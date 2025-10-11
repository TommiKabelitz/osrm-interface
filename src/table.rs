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
    pub sources: &'a [Point],
    pub destinations: &'a [Point],
    pub annotations: TableAnnotation,
    pub fallback_speed: Option<f64>,
    pub fallback_coordinate: Option<TableFallbackCoordinate>,
    pub scale_factor: Option<f64>,
}

#[derive(Debug)]
pub enum TableRequestError {
    EmptySources,
    EmptyDestinations,
    NonPositiveValue(&'static str),
}

pub struct TableRequestBuilder<'a> {
    sources: &'a [Point],
    destinations: &'a [Point],
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
                return Err(TableRequestError::NonPositiveValue("fallback_speed"));
            }
        }
        #[allow(clippy::collapsible_if)]
        if let Some(f) = self.scale_factor {
            if f <= 0.0 {
                return Err(TableRequestError::NonPositiveValue("scale_factor"));
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

pub enum TableReqestError {
    NonPositiveValue,
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
