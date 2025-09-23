use std::vec;

use crate::algorithm;
use crate::errors::OsrmError;
use crate::point::Point;
use crate::route::{Leg, Route, RouteRequest, RouteResponse, SimpleRouteResponse, Step};
use crate::tables::{TableLocationEntry, TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};
use crate::waypoints::Waypoint;

pub struct OsrmEngine {}

impl OsrmEngine {
    pub fn new(_base_path: &str, _algorithm: algorithm::Algorithm) -> Result<Self, OsrmError> {
        Ok(Self {})
    }

    pub fn table(&self, table_request: TableRequest) -> Result<TableResponse, OsrmError> {
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }

        let durations: Vec<Vec<Option<f64>>> = (0..len_sources)
            .map(|i| {
                (0..len_destinations)
                    .map(|j| Some(if i == j { 0.0 } else { 1.0 }))
                    .collect()
            })
            .collect();

        Ok(TableResponse {
            code: "Ok".to_string(),
            destinations: table_request
                .destinations
                .iter()
                .map(|p| TableLocationEntry {
                    hint: "Mocked hint".to_string(),
                    location: [p.latitude(), p.longitude()],
                    name: "Mocked_name".to_string(),
                    distance: 0.0,
                })
                .collect(),
            sources: table_request
                .sources
                .iter()
                .map(|p| TableLocationEntry {
                    hint: "Mock hint".to_string(),
                    location: [p.latitude(), p.longitude()],
                    name: "Mock name".to_string(),
                    distance: 0.0,
                })
                .collect(),
            durations,
        })
    }

    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        let len = route_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }

        let routes: Vec<Route> = route_request
            .points
            .windows(2)
            .map(|_| Route {
                legs: vec![Leg {
                    steps: vec![Step {}],
                    weight: 0.1,
                    summary: "Mock summary".to_string(),
                    duration: 1.0,
                    distance: 1.0,
                }],
                weight_name: "Mock weight".to_string(),
                geometry: "Mock polyline".to_string(),
                weight: 0.1,
                duration: 1.0,
            })
            .collect();

        Ok(RouteResponse {
            code: "Ok".to_string(),
            routes,
            waypoints: route_request
                .points
                .iter()
                .map(|p| Waypoint {
                    hint: "Mock hint".to_string(),
                    location: [p.latitude(), p.longitude()],
                    name: "Mock name".to_string(),
                    distance: 0.0,
                })
                .collect(),
        })
    }

    pub fn trip(&self, trip_request: TripRequest) -> Result<TripResponse, OsrmError> {
        let len = trip_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }
        Ok(TripResponse {})
    }

    pub fn simple_route(&self, _from: Point, _to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        Ok(SimpleRouteResponse {
            code: "Ok".to_string(),
            distance: 0.0,
            durations: 1.0,
        })
    }
}
