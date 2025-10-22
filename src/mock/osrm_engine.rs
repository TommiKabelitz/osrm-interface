use crate::algorithm;
use crate::errors::OsrmError;
use crate::osrm_response_types::{Route, Waypoint};
use crate::point::Point;
use crate::route::{RouteRequest, RouteResponse, SimpleRouteResponse};
use crate::table::{TableAnnotation, TableLocationEntry, TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

pub struct OsrmEngine {}

impl OsrmEngine {
    pub fn new(_base_path: &str, _algorithm: algorithm::Algorithm) -> Self {
        Self {}
    }

    pub fn table(&self, table_request: TableRequest) -> Result<TableResponse, OsrmError> {
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableRequest);
        }

        // Just lazily create both even if we don't need them
        // because it is just the mocking function
        let durations: Vec<Vec<Option<f64>>> = (0..len_sources)
            .map(|i| {
                (0..len_destinations)
                    .map(|j| Some(if i == j { 0.0 } else { 1.0 }))
                    .collect()
            })
            .collect();

        let distances: Vec<Vec<Option<f64>>> = (0..len_sources)
            .map(|i| {
                (0..len_destinations)
                    .map(|j| Some(if i == j { 0.0 } else { 2.0 }))
                    .collect()
            })
            .collect();

        let (durations, distances) = match table_request.annotations {
            TableAnnotation::All => (Some(durations), Some(distances)),
            TableAnnotation::Distance => (None, Some(distances)),
            TableAnnotation::Duration => (Some(durations), None),
            TableAnnotation::None => (None, None),
        };

        Ok(TableResponse {
            code: "Ok".to_string(),
            destinations: table_request
                .destinations
                .iter()
                .map(|p| TableLocationEntry {
                    hint: Some("Mock hint".to_string()),
                    location: [p.latitude(), p.longitude()],
                    name: "Mock name".to_string(),
                    distance: 0.0,
                })
                .collect(),
            sources: table_request
                .sources
                .iter()
                .map(|p| TableLocationEntry {
                    hint: Some("Mock hint".to_string()),
                    location: [p.latitude(), p.longitude()],
                    name: "Mock name".to_string(),
                    distance: 0.0,
                })
                .collect(),
            durations,
            distances,
        })
    }

    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        if route_request.points.len() < 2 {
            return Err(OsrmError::InvalidRouteRequest);
        }

        let routes: Vec<Route> = route_request
            .points
            .windows(2)
            .map(|_| Route::default())
            .collect();

        Ok(RouteResponse {
            code: "Ok".to_string(),
            routes,
            waypoints: Some(
                route_request
                    .points
                    .iter()
                    .map(|p| Waypoint {
                        hint: "Mock hint".to_string(),
                        location: [p.latitude(), p.longitude()],
                        name: "Mock name".to_string(),
                        distance: 0.0,
                    })
                    .collect(),
            ),
        })
    }

    pub fn trip(&self, trip_request: TripRequest) -> Result<TripResponse, OsrmError> {
        if trip_request.points.len() < 2 {
            return Err(OsrmError::InvalidRouteRequest);
        }
        let trips: Vec<Route> = trip_request
            .points
            .windows(2)
            .map(|_| Route::default())
            .collect();

        Ok(TripResponse {
            code: "Ok".to_string(),
            trips,
            waypoints: trip_request
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

    pub fn simple_route(&self, _from: Point, _to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        Ok(SimpleRouteResponse {
            code: "Ok".to_string(),
            distance: 0.0,
            durations: 1.0,
        })
    }
}
