use thiserror::Error;

use crate::{
    r#match::Approach,
    osrm_response_types::Waypoint,
    point::Point,
    request_types::{Bearing, Exclude, Snapping},
};

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct NearestRequest<'a> {
    pub(crate) point: &'a Point,
    pub(crate) number: u64,
    pub(crate) bearing: Option<Bearing>,
    pub(crate) radius: Option<f64>,
    pub(crate) approach: Option<Approach>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
}

pub struct NearestRequestBuilder<'a> {
    pub point: &'a Point,
    pub number: u64,
    bearing: Option<Bearing>,
    radius: Option<f64>,
    approach: Option<Approach>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
}

impl<'a> NearestRequestBuilder<'a> {
    pub fn new(point: &'a Point, number: u64) -> Self {
        Self {
            point,
            number,
            bearing: None,
            radius: None,
            approach: None,
            exclude: None,
            snapping: None,
        }
    }
    pub fn bearing(mut self, bearing: Bearing) -> Self {
        self.bearing = Some(bearing);
        self
    }

    pub fn radius(mut self, coordinate_radius: f64) -> Self {
        self.radius = Some(coordinate_radius);
        self
    }

    pub fn approach(mut self, approach_direction: Approach) -> Self {
        self.approach = Some(approach_direction);
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

    pub fn build(self) -> Result<NearestRequest<'a>, NearestRequestError> {
        #[allow(clippy::collapsible_if)]
        if let Some(exclude) = self.exclude {
            if !exclude.is_empty() {
                if !match exclude[0] {
                    Exclude::Car(_) => exclude.iter().all(|e| matches!(e, Exclude::Car(_))),
                    Exclude::Bicycle(_) => exclude.iter().all(|e| matches!(e, Exclude::Bicycle(_))),
                } {
                    return Err(NearestRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(NearestRequest {
            point: self.point,
            number: self.number,
            bearing: self.bearing,
            radius: self.radius,
            approach: self.approach,
            exclude: self.exclude,
            snapping: self.snapping,
        })
    }
}
#[derive(Error, Debug)]
pub enum NearestRequestError {
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct NearestResponse {
    /// If the request was successful "Ok" otherwise see the service dependent and general status codes.
    pub code: String,
    /// Array of Waypoint objects sorted by distance to the input coordinate
    pub waypoints: Vec<Waypoint>,
}
