#![cfg(feature = "native")]

mod common;
use common::init_native_engine;

use osrm_interface::{point::Point, route::RouteRequest};
use rand::Rng;

#[test]
fn test_basic_native_route() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
        Point::new(48.942296, 10.510960).expect("Invalid point"),
        Point::new(51.248931, 7.594814).expect("Invalid point"),
    ];
    let route_request = RouteRequest::new(&points).expect("No points in request");

    let response = engine
        .route(&route_request)
        .expect("Failed to route request");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.routes.iter().map(|r| r.legs.len()).sum::<usize>(),
        3,
        "Route response length is incorrect"
    )
}

#[test]
fn test_two_point_native_route() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
    ];
    let route_request = RouteRequest::new(&points).expect("No points in request");

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
#[ignore]
fn test_large_native_route() {
    let engine = init_native_engine(".env");

    let mut rng = rand::rng();
    let points = (0..1_000)
        .map(|_| Point::new(rng.random_range(49.0..53.0), rng.random_range(8.3..12.0)).unwrap())
        .collect::<Vec<_>>();

    let route_request = RouteRequest::new(&points).expect("No points in request");

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
