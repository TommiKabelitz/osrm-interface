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

impl<'a> TableRequest<'a> {
    pub fn new(sources: &'a [Point], destinations: &'a [Point]) -> Option<Self> {
        if sources.is_empty() || destinations.is_empty() {
            return None;
        }
        Some(Self {
            sources,
            destinations,
            annotations: TableAnnotation::Duration,
            fallback_speed: None,
            fallback_coordinate: None,
            scale_factor: None,
        })
    }

    pub fn with_annotations(mut self, annotations: TableAnnotation) -> Self {
        self.annotations = annotations;
        self
    }

    pub fn with_fallback(
        mut self,
        fallback_coordinate: TableFallbackCoordinate,
        fallback_speed: f64,
    ) -> Result<Self, (Self, TableReqestError)> {
        if fallback_speed <= 0.0 {
            return Err((self, TableReqestError::NonPositiveValue));
        }
        self.fallback_coordinate = Some(fallback_coordinate);
        self.fallback_speed = Some(fallback_speed);
        Ok(self)
    }

    pub fn with_scale_factor(
        mut self,
        scale_factor: f64,
    ) -> Result<Self, (Self, TableReqestError)> {
        if scale_factor <= 0.0 {
            return Err((self, TableReqestError::NonPositiveValue));
        }
        self.scale_factor = Some(scale_factor);
        Ok(self)
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
