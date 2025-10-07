use crate::osrm_response_types::Waypoint;

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
