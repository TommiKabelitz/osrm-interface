use std::{fmt::Debug, io::BufRead};

use osrm_interface::{
    osrm_response_types::Geometry,
    point::Point,
    request_types::{GeometryType, OverviewZoom},
    route::RouteRequest,
};

fn main() {
    #[cfg(feature = "native")]
    {
        let engine = init_native_engine(".env");

        let points = [
            Point::new(48.040437, 10.316550).expect("Invalid point"),
            Point::new(49.006101, 9.052887).expect("Invalid point"),
        ];
        let route_request = RouteRequest::new(&points)
            .expect("No points in request")
            .with_geometry(GeometryType::GeoJSON)
            .with_overview(OverviewZoom::Full);

        let response = engine
            .route(&route_request)
            .expect("Failed to route request");

        assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
        assert!(
            matches!(
                response.routes.first().unwrap().geometry,
                Some(Geometry::GeoJson(_))
            ),
            "Geometry should be GeoJson"
        );

        let route_request = RouteRequest::new(&points)
            .expect("No points in request")
            .with_geometry(GeometryType::Polyline6)
            .with_overview(OverviewZoom::Full);

        let response = engine
            .route(&route_request)
            .expect("Failed to route request");

        assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
        assert!(
            matches!(
                response.routes.first().unwrap().geometry,
                Some(Geometry::Polyline(_))
            ),
            "Geometry should be Polyline"
        );
    }

    #[cfg(feature = "remote")]
    {
        let engine = init_remote_engine(".env");

        let points = [
            Point::new(48.040437, 10.316550).expect("Invalid point"),
            Point::new(49.006101, 9.052887).expect("Invalid point"),
        ];
        let route_request = RouteRequest::new(&points)
            .expect("No points in request")
            .with_geometry(GeometryType::GeoJSON)
            .with_overview(OverviewZoom::Full);

        let response = engine
            .route(&route_request)
            .expect("Failed to route request");

        assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
        assert!(
            matches!(
                response.routes.first().unwrap().geometry,
                Some(Geometry::GeoJson(_))
            ),
            "Geometry should be GeoJson"
        );

        let route_request = RouteRequest::new(&points)
            .expect("No points in request")
            .with_geometry(GeometryType::Polyline6)
            .with_overview(OverviewZoom::Full);

        let response = engine
            .route(&route_request)
            .expect("Failed to route request");

        assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
        assert!(
            matches!(
                response.routes.first().unwrap().geometry,
                Some(Geometry::Polyline(_))
            ),
            "Geometry should be Polyline"
        );
    }
}

#[cfg(feature = "native")]
#[allow(dead_code)]
pub fn init_native_engine(dotenv_path: &str) -> osrm_interface::native::OsrmEngine {
    let osrm_map_file = load_dotenv_value(dotenv_path, "OSRM_MAP_FILE")
        .expect("Failed to load .env which needs to set OSRM_MAP_FILE for native tests");
    osrm_interface::native::OsrmEngine::new(
        &osrm_map_file,
        osrm_interface::algorithm::Algorithm::MLD,
    )
    .expect("Failed to init native OSRM engine")
}

#[cfg(feature = "remote")]
#[allow(dead_code)]
pub fn init_remote_engine(dotenv_path: &str) -> osrm_interface::remote::OsrmEngine {
    let endpoint = load_dotenv_value(dotenv_path, "OSRM_ROUTED_ADDRESS")
        .expect("Failed to load .env which needs to set OSRM_ROUTED_ADDRESS for remote tests");
    osrm_interface::remote::OsrmEngine::new(
        endpoint.to_string(),
        osrm_interface::request_types::Profile::Car,
    )
}

/// Load a value from a .env
///
/// Scans through the file looking for a line `key=value`
pub fn load_dotenv_value(path: &str, key: &str) -> Result<String, DotEnvError> {
    let file = std::fs::File::open(path).map_err(DotEnvError::DotEnvNotFound)?;
    let reader = std::io::BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(DotEnvError::Other)?; // unwrap Result
        let line = line.trim();
        if line.starts_with(key) {
            return line
                .split_once("=")
                .ok_or(DotEnvError::SyntaxError(i))
                .map(|(_, v)| v.to_string());
        }
    }
    Err(DotEnvError::KeyNotFound)
}

pub enum DotEnvError {
    DotEnvNotFound(std::io::Error),
    SyntaxError(usize),
    KeyNotFound,
    Other(std::io::Error),
}

impl Debug for DotEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DotEnvNotFound(e) => f.debug_struct("DotEnvNotFound").field("error", &e).finish(),
            Self::SyntaxError(l) => f
                .debug_struct("SyntaxError")
                .field("line_number", &l)
                .finish(),
            Self::KeyNotFound => f.debug_struct("KeyNotFound").finish(),
            Self::Other(e) => f.debug_struct("Other").field("error", &e).finish(),
        }
    }
}
