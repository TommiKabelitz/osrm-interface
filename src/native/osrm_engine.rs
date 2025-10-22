use crate::algorithm;
use crate::errors::{NativeOsrmError, OsrmError};
use crate::r#match::{MatchRequest, MatchResponse};
use crate::native::Osrm;
use crate::nearest::{NearestRequest, NearestResponse};
use crate::point::Point;
use crate::route::{RouteRequest, RouteRequestBuilder, RouteResponse, SimpleRouteResponse};
use crate::table::{TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

pub struct OsrmEngine {
    instance: Osrm,
}
use std::str;
impl OsrmEngine {
    pub fn new(base_path: &str, algorithm: algorithm::Algorithm) -> Result<Self, OsrmError> {
        let osrm = Osrm::new(base_path, algorithm.as_str())
            .map_err(|e| OsrmError::Native(NativeOsrmError::Initialization(e)))?;
        Ok(OsrmEngine { instance: osrm })
    }

    pub fn table(&self, table_request: &TableRequest) -> Result<TableResponse, OsrmError> {
        let result = self
            .instance
            .table(table_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<TableResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        let len = route_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidRouteRequest);
        }
        let result = self
            .instance
            .route(route_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<RouteResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    pub fn trip(&self, trip_request: TripRequest) -> Result<TripResponse, OsrmError> {
        let len = trip_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTripRequest);
        }
        let coordinates: &[(f64, f64)] = &trip_request
            .points
            .iter()
            .map(|p| (p.longitude(), p.latitude()))
            .collect::<Vec<(f64, f64)>>()[..];
        let result = self
            .instance
            .trip(coordinates)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        serde_json::from_str::<TripResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))
    }

    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let points = [from, to];
        let request = RouteRequestBuilder::new(&points)
            .build()
            .expect("Route request for simple route is empty");

        let result = self
            .instance
            .route(&request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let route_response = serde_json::from_str::<RouteResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;
        if route_response.routes.is_empty() {
            return Err(OsrmError::EmptyResponse(
                "No route was returned between those 2 points".to_owned(),
            ));
        }
        Ok(SimpleRouteResponse {
            code: route_response.code,
            distance: route_response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.distance)
                .sum(),
            durations: route_response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.duration)
                .sum(),
        })
    }

    pub fn nearest(&self, nearest_request: &NearestRequest) -> Result<NearestResponse, OsrmError> {
        let result = self
            .instance
            .nearest(nearest_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let nearest_response = serde_json::from_str::<NearestResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;
        Ok(nearest_response)
    }

    pub fn r#match(&self, match_request: &MatchRequest) -> Result<MatchResponse, OsrmError> {
        if match_request.points.is_empty() {
            return Err(OsrmError::InvalidMatchRequest);
        }

        let result = self
            .instance
            .r#match(match_request)
            .map_err(|e| OsrmError::Native(NativeOsrmError::FfiError(e)))?;
        let response = serde_json::from_str::<MatchResponse>(&result)
            .map_err(|e| OsrmError::Native(NativeOsrmError::JsonParse(Box::new(e))))?;

        Ok(response)
    }
}
