#![cfg(feature = "remote")]

mod common;
use common::init_remote_engine;

use osrm_interface::{
    osrm_response_types::Geometry,
    point::Point,
    request_types::{GeometryType, OverviewZoom},
    route::RouteRequest,
    tables::TableRequest,
    trip::TripRequest,
};

#[test]
fn test_basic_remote_route() {
    let engine = init_remote_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 10.052887).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
        Point::new(48.942296, 10.510960).expect("Invalid point"),
        Point::new(51.248931, 7.594814).expect("Invalid point"),
    ];
    let route_request = RouteRequest::new(&points)
        .expect("No points in request")
        .with_geometry(osrm_interface::request_types::GeometryType::GeoJSON);

    let response = engine
        .route(&route_request)
        .expect("Failed to route request");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.routes.iter().map(|r| r.legs.len()).sum::<usize>(),
        points.len() - 1,
        "Route response length is incorrect"
    )
}

#[test]
fn test_basic_remote_trip() {
    let engine = init_remote_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
        Point::new(48.942296, 10.510960).expect("Invalid point"),
        Point::new(51.248931, 7.594814).expect("Invalid point"),
    ];
    let trip_request = TripRequest::new(&points)
        .expect("No points in trip request")
        .with_geometry(osrm_interface::request_types::GeometryType::GeoJSON);

    let trip_response = engine.trip(&trip_request).expect("Failed navigate trip");

    assert_eq!(trip_response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        trip_response
            .trips
            .iter()
            .map(|r| r.legs.len())
            .sum::<usize>(),
        points.len(),
        "Route response length is incorrect"
    )
}

#[test]
fn test_remote_route_geometries() {
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

#[test]
fn test_remote_nearest() {
    let engine = init_remote_engine(".env");

    let num_points = 3;
    let point = Point::new(48.040437, 10.316550).expect("Invalid point");
    let response = engine
        .nearest(&point, num_points)
        .expect("Failed to find nearest");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.waypoints.len(),
        num_points as usize,
        "Nearest returned the wrong number of points"
    );
}

#[test]
fn test_remote_table() {
    let engine = init_remote_engine(".env");

    let sources = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.040437, 10.216550).expect("Invalid point"),
    ];
    let destinations = [
        Point::new(48.540437, 10.816550).expect("Invalid point"),
        Point::new(49.140437, 10.416550).expect("Invalid point"),
        Point::new(49.140437, 10.516550).expect("Invalid point"),
    ];
    let table_request =
        TableRequest::new(&sources, &destinations).expect("Failed to create table request");
    let response = engine
        .table(table_request)
        .expect("Failed to determine table");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.sources.len(),
        sources.len(),
        "Returned more sources than anticipated"
    );
    assert_eq!(
        response.destinations.len(),
        destinations.len(),
        "Returned more destinations than anticipated"
    );
    assert!(
        response.distances.is_none(),
        "Distances should be None by default"
    );
    assert!(
        response.durations.is_some(),
        "Durations should be Some by default"
    );
}

#[test]
fn test_table_options() {
    let engine = init_remote_engine(".env");

    let sources = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.040437, 10.216550).expect("Invalid point"),
    ];
    let destinations = [
        Point::new(48.540437, 10.816550).expect("Invalid point"),
        Point::new(49.140437, 10.416550).expect("Invalid point"),
        Point::new(49.140437, 10.516550).expect("Invalid point"),
    ];
    let table_request = TableRequest::new(&sources, &destinations)
        .expect("Failed to create table request")
        .with_annotations(osrm_interface::tables::TableAnnotation::All);
    let response = engine
        .table(table_request)
        .expect("Failed to determine table");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.sources.len(),
        sources.len(),
        "Returned more sources than anticipated"
    );
    assert_eq!(
        response.destinations.len(),
        destinations.len(),
        "Returned more destinations than anticipated"
    );
    assert!(response.distances.is_some(), "Distances should be Some");
    assert!(response.durations.is_some(), "Durations should be Some");
}
