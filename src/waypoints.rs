#[derive(Debug)]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Waypoint {
    pub hint: String,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
}
