use crate::Point;
use crate::errors::OsrmError;
use crate::r#match::{MatchRequest, MatchResponse};
use crate::nearest::{NearestRequest, NearestResponse};
use crate::osrm_response_types::{MatchRoute, MatchWaypoint, Route, TripWaypoint, Waypoint};
use crate::route::{RouteRequest, RouteResponse, SimpleRouteResponse};
use crate::table::{TableAnnotation, TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

/// The engine for calling into the mocked osrm-backend.
///
/// The mock engine returns data of the appropriate type,
/// but all data is fabricated.
pub struct OsrmEngine {}

impl OsrmEngine {
    /// Initialise the mock engine.
    ///
    /// The mock engine returns data of the appropriate type, but
    /// all data is fabricated.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    /// Given a set of source and destination `Point`s or `Hint`s, determine the distances
    /// and/or durations to travel between all sources and destinations.
    ///
    /// See `TableRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Computes the duration of the fastest route between all pairs of supplied coordinates.
    /// Returns durations or distances or both between the coordinate pairs. Note that the
    /// distances are not the shortest distance between two coordinates, but rather the
    /// distances of the fastest routes. Durations are in seconds and distances are in meters.
    pub fn table(&self, table_request: TableRequest) -> Result<TableResponse, OsrmError> {
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();

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
            destinations: Some(
                table_request
                    .destinations
                    .iter()
                    .map(|p| Waypoint {
                        hint: Some("Mock hint".to_string()),
                        location: [p.latitude(), p.longitude()],
                        name: "Mock name".to_string(),
                        distance: 0.0,
                    })
                    .collect(),
            ),
            sources: Some(
                table_request
                    .sources
                    .iter()
                    .map(|p| Waypoint {
                        hint: Some("Mock hint".to_string()),
                        location: [p.latitude(), p.longitude()],
                        name: "Mock name".to_string(),
                        distance: 0.0,
                    })
                    .collect(),
            ),
            durations,
            distances,
            fallback_speed_cells: None,
        })
    }

    /// Given an ordered set of `Point`s or `Hint`s, route through those points in the
    /// given order.
    ///
    /// See `RouteRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Finds the fastest route between coordinates in the supplied order.
    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
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
                        hint: Some("Mock hint".to_string()),
                        location: [p.latitude(), p.longitude()],
                        name: "Mock name".to_string(),
                        distance: 0.0,
                    })
                    .collect(),
            ),
        })
    }

    /// Given an _unordered_ set of `Point`s or `Hint`s, uses a greedy heuristic to
    /// approximately solve the travelling salesman problem. Returns the fastest route
    /// through those points in some order.
    ///
    /// See `TripRequest` for all possible options.
    pub fn trip(&self, trip_request: TripRequest) -> Result<TripResponse, OsrmError> {
        let trips: Vec<Route> = trip_request
            .points
            .windows(2)
            .map(|_| Route::default())
            .collect();

        Ok(TripResponse {
            code: "Ok".to_string(),
            trips,
            waypoints: Some(
                trip_request
                    .points
                    .iter()
                    .enumerate()
                    .map(|(i, p)| TripWaypoint {
                        hint: Some("Mock hint".to_string()),
                        location: [p.latitude(), p.longitude()],
                        name: "Mock name".to_string(),
                        distance: 0.0,
                        trips_index: i,
                        waypoint_index: i,
                    })
                    .collect(),
            ),
        })
    }

    /// A massively simplified interface for routing just between two points.
    ///
    /// Calls OsrmEngine::route with default options.
    pub fn simple_route(&self, _from: Point, _to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        Ok(SimpleRouteResponse {
            code: "Ok".to_string(),
            distance: 0.0,
            durations: 1.0,
        })
    }

    /// Snap the given `Point` to the n closest nodes on the map. Returning the snapped
    /// coordinates and various metrics.
    ///
    /// `Hint`s returned from `nearest` may be passed to other services, allowing
    /// the call to skip the snapping process on subsequent calls.
    ///
    /// See `NearestRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Snaps a coordinate to the street network and returns the nearest n matches.
    pub fn nearest(&self, nearest_request: &NearestRequest) -> Result<NearestResponse, OsrmError> {
        let point = nearest_request.point;
        Ok(NearestResponse {
            code: "Ok".to_string(),
            waypoints: vec![Waypoint {
                hint: Some("Mock hint".to_string()),
                location: [point.latitude(), point.longitude()],
                name: "Mock name".to_string(),
                distance: 0.0,
            }],
        })
    }

    /// Given an ordered set of `Point`s or `Hint`s (and optionally
    /// timestamps), determine the likely route taken that could match
    /// those coordinates. Returns the route and confidence values.
    ///
    /// See `MatchRequest` for all possible options.
    ///
    /// ## Official documentation
    ///
    /// Map matching matches/snaps given GPS points to the road network
    /// in the most plausible way. Please note the request might result
    /// in multiple sub-traces. Large jumps in the timestamps (> 60s) or
    /// improbable transitions lead to trace splits if a complete matching
    /// could not be found. The algorithm might not be able to match all
    /// points. Outliers are removed if they can not be matched successfully.
    pub fn r#match(&self, match_request: &MatchRequest) -> Result<MatchResponse, OsrmError> {
        let matchings: Vec<MatchRoute> = match_request
            .points
            .windows(2)
            .map(|_| MatchRoute::default())
            .collect();

        Ok(MatchResponse {
            code: "Ok".to_string(),
            matchings,
            tracepoints: match_request
                .points
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    Some(MatchWaypoint {
                        hint: "Mock hint".to_string(),
                        location: [p.latitude(), p.longitude()],
                        name: "Mock name".to_string(),
                        distance: 0.0,
                        matchings_index: 0,
                        waypoint_index: i as u64,
                        alternatives_count: 0,
                    })
                })
                .collect(),
        })
    }
}
