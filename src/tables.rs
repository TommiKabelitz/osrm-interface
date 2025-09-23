pub(crate) use crate::point::Point;
use derive_builder::Builder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TableResponse {
    pub code: String,
    pub destinations: Vec<TableLocationEntry>,
    pub durations: Vec<Vec<Option<f64>>>,
    pub sources: Vec<TableLocationEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TableLocationEntry {
    pub hint: String,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
}

#[derive(Debug, Builder, Clone)]
pub struct TableRequest {
    pub sources: Vec<Point>,
    pub destinations: Vec<Point>,
}
