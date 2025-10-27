//! Snap a (latitude, longitude) coordinate to the nearest node
//! on the map.

use thiserror::Error;

use crate::{
    Point,
    osrm_response_types::Waypoint,
    request_types::{Bearing, Exclude, Snapping},
    services::Approach,
};

/// The request object passed to the nearest service. Constructed
/// through [`NearestRequestBuilder::build`] which verifies the
/// validity of the request.
///
/// See [`NearestRequestBuilder`] for more information on nearest requests.
#[derive(Clone, Debug)]
pub struct NearestRequest<'a> {
    pub(crate) point: &'a Point,
    pub(crate) number: u64,
    pub(crate) bearing: Option<Bearing>,
    pub(crate) radius: Option<f64>,
    pub(crate) approach: Option<Approach>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
}

/// Helper struct for building a [`NearestRequest`].
///
/// Set nearest options using the struct methods before calling
/// [`build`](Self::build). The `build` method validates the configuration
/// and attempts to detect invalid combinations before the request is sent
/// to the service.
///
/// ## Options
///
/// - **`point`** (*required*) — The [`Point`] coordinate for which nearest
///   road segments are queried.
///
/// - **`number`** (*required*) — The maximum number of nearest segments to return.
///
/// - **`bearing`** (*optional*) — A [`Bearing`] restricting the direction
///   in which the coordinate may be snapped to a road segment.
///
/// - **`radius`** (*optional*) — A search radius (in meters) constraining how far
///   OSRM may search from the input coordinate. If omitted, the search radius
///   is unlimited.
///
/// - **`approach`** (*optional*) — The [`Approach`] side of the road from which
///   to approach the coordinate (e.g., `Approach::Curb`, `Approach::Unrestricted`).
///   The default is unrestricted access.
///
/// - **`exclude`** (*optional*) — A slice of [`Exclude`] values, all of the same
///   transport mode (e.g., all `Exclude::Car` or all `Exclude::Bicycle`),
///   specifying road classes to exclude from the search.
///
/// - **`snapping`** (*optional*) — Defines how the input coordinate is snapped to
///   the road network. See [`Snapping`] for available modes.
///
/// ## Example
///
/// ```
/// use osrm_interface::{Point, nearest::NearestRequestBuilder, request_types::Bearing};
/// let point = Point::new(48.040437, 10.316550).expect("Invalid point");
/// let nearest_request = NearestRequestBuilder::new(&point, 3)
///     .radius(50.0)
///     // The actual node may be 20° either side of north of the given point
///     .bearing(Bearing::new(0, 20).unwrap())
///     .build()
///     .expect("Failed to build NearestRequest");
/// ```
#[derive(Clone, Debug)]
pub struct NearestRequestBuilder<'a> {
    point: &'a Point,
    number: u64,
    bearing: Option<Bearing>,
    radius: Option<f64>,
    approach: Option<Approach>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
}

impl<'a> NearestRequestBuilder<'a> {
    /// Creates a new [`NearestRequestBuilder`] with default parameters.
    ///
    /// The builder can then be customized using its setter methods.
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

    /// Overwrite the point provided at construction of the builder. Useful
    /// for reusing a builder with the same options.
    pub fn point(mut self, point: &'a Point) -> Self {
        self.point = point;
        self
    }

    /// Sets the bearing to constrain the direction snapping to
    /// the node.
    pub fn bearing(mut self, bearing: Bearing) -> Self {
        self.bearing = Some(bearing);
        self
    }

    /// Sets the search radius (in meters) constraining how far OSRM may
    /// search from the input coordinate.
    ///
    /// Radius must be positive. Passing None corresponds to an infinite
    /// search radius.
    pub fn radius(mut self, coordinate_radius: f64) -> Self {
        self.radius = Some(coordinate_radius);
        self
    }

    /// Sets the approach direction to control the side of the road
    /// from which the coordinate is accessed.
    pub fn approach(mut self, approach_direction: Approach) -> Self {
        self.approach = Some(approach_direction);
        self
    }

    /// Sets which road classes should be excluded from the nearest search.
    ///
    /// All excludes must belong to the same transport mode.
    pub fn exclude(mut self, exclude: &'a [Exclude]) -> Self {
        self.exclude = Some(exclude);
        self
    }

    /// Sets the snapping behavior for the input coordinate.
    pub fn snapping(mut self, snapping: Snapping) -> Self {
        self.snapping = Some(snapping);
        self
    }

    /// Builds a [`NearestRequest`] from the configured parameters.
    ///
    /// Performs validation to ensure all parameters are compatible.
    ///
    /// # Errors
    ///
    /// Returns a [`NearestRequestError`] if:
    /// - Exclude types mix transport modes.
    /// - Radius is negative.
    pub fn build(&self) -> Result<NearestRequest<'a>, NearestRequestError> {
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

        #[allow(clippy::collapsible_if)]
        if let Some(r) = self.radius {
            if r < 0.0 {
                return Err(NearestRequestError::NegativeRadius);
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

/// The comprehensive error type returned when attempting to
/// construct an invalid [`NearestRequest`].
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Error, Debug)]
pub enum NearestRequestError {
    /// Cannot mix excludes of different [`Exclude`] variants.
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
    /// Radius values must be non-negative.
    #[error("Radii must be non-negative")]
    NegativeRadius,
}

/// The response type returned by the Nearest service.
///
/// Implements [`serde::Deserialize`] and
/// [`serde::Serialize`] if `feature="serde"` is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug)]
pub struct NearestResponse {
    /// If the request was successful "Ok" otherwise see the service dependent and general status codes.
    pub code: String,
    /// Array of Waypoint objects sorted by distance to the input coordinate
    pub waypoints: Vec<Waypoint>,
}
