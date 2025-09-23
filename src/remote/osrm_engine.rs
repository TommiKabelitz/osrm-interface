use itertools::Itertools;

use crate::algorithm;
use crate::errors::OsrmError;
use crate::point::Point;
use crate::route::{RouteRequest, RouteResponse, SimpleRouteResponse};
use crate::tables::{TableRequest, TableResponse};
use crate::trip::{TripRequest, TripResponse};

pub struct OsrmEngine {
    endpoint: &'static str,
}

impl OsrmEngine {
    pub fn new(_base_path: &str, _algorithm: algorithm::Algorithm) -> Result<Self, OsrmError> {
        let endpoint = env!("OSRM_ENDPOINT_COMPILED");
        Ok(Self { endpoint })
    }

    pub fn table(&self, table_request: TableRequest) -> Result<TableResponse, OsrmError> {
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();
        if len_sources == 0 || len_destinations == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }

        let coordinates = table_request
            .sources
            .iter()
            .chain(table_request.destinations.iter())
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let source_indices = (0..len_sources).map(|i| format!("{}", i)).join(",");
        let destination_indices = (len_sources..(len_sources + len_destinations))
            .map(|i| format!("{}", i))
            .join(";");

        let url = format!(
            "{}/table/v1/driving/{coordinates}?sources={source_indices}&destinations={destination_indices}",
            self.endpoint
        );
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::EndpointError(e.to_string()))?
            .into_body()
            .read_json::<TableResponse>()
            .map_err(|e| OsrmError::EndpointError(e.to_string()))?;

        Ok(response)
    }

    pub fn route(&self, route_request: &RouteRequest) -> Result<RouteResponse, OsrmError> {
        let len = route_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }
        let coordinates = route_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let url = format!(
            "http://{}/route/v1/driving/{coordinates}?geometries=polyline&overview=full",
            self.endpoint
        );
        println!("URL: {url}");
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::EndpointError(e.to_string()))?
            .into_body()
            .read_json::<RouteResponse>()
            .map_err(|e| OsrmError::EndpointError(e.to_string()))?;

        Ok(response)
    }

    pub fn trip(&self, trip_request: TripRequest) -> Result<TripResponse, OsrmError> {
        let len = trip_request.points.len();
        if len == 0 {
            return Err(OsrmError::InvalidTableArgument);
        }
        let coordinates = trip_request
            .points
            .iter()
            .map(|p| format!("{:.6},{:.6}", p.longitude(), p.latitude()))
            .join(";");

        let url = format!(
            "{}/trip/v1/driving/{coordinates}?geometries=polyline&overview=full",
            self.endpoint
        );
        let response = ureq::get(url)
            .call()
            .map_err(|e| OsrmError::EndpointError(e.to_string()))?
            .into_body()
            .read_json::<TripResponse>()
            .map_err(|e| OsrmError::EndpointError(e.to_string()))?;

        Ok(response)
    }

    pub fn simple_route(&self, from: Point, to: Point) -> Result<SimpleRouteResponse, OsrmError> {
        let points = [from, to];
        let full_request =
            RouteRequest::new(&points).expect("Route request for simple route is empty");
        let response = self.route(&full_request)?;

        Ok(SimpleRouteResponse {
            code: response.code,
            distance: response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.distance)
                .sum(),
            durations: response
                .routes
                .first()
                .unwrap()
                .legs
                .iter()
                .map(|l| l.duration)
                .sum(),
        })
    }
}
