use crate::osrm_response_types::Waypoint;

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct NearestResponse {
    pub code: String,
    pub waypoints: Vec<Waypoint>,
}
