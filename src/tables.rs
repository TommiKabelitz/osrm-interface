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
}

impl<'a> TableRequest<'a> {
    pub fn new(sources: &'a [Point], destinations: &'a [Point]) -> Option<Self> {
        if sources.is_empty() || destinations.is_empty() {
            return None;
        }
        Some(Self {
            sources,
            destinations,
        })
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct TableResponse {
    pub code: String,
    pub destinations: Vec<TableLocationEntry>,
    pub durations: Vec<Vec<Option<f64>>>,
    pub sources: Vec<TableLocationEntry>,
}
