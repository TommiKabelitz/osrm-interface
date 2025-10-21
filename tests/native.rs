#![cfg(feature = "native")]

mod common;
use std::f64::consts::PI;

use common::init_native_engine;

use osrm_interface::{
    r#match::MatchRequestBuilder,
    osrm_response_types::Geometry,
    point::Point,
    request_types::{Bearing, CarExclude, Exclude, GeometryType, OverviewZoom},
    route::RouteRequestBuilder,
    table::TableRequestBuilder,
    trip::TripRequestBuilder,
};
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
    let route_request = RouteRequestBuilder::new(&points)
        .build()
        .expect("No points in request");

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
    let route_request = RouteRequestBuilder::new(&points)
        .build()
        .expect("No points in request");

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

    let route_request = RouteRequestBuilder::new(&points)
        .build()
        .expect("No points in request");

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
fn test_basic_native_trip() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
        Point::new(48.942296, 10.510960).expect("Invalid point"),
        Point::new(51.248931, 7.594814).expect("Invalid point"),
    ];
    let trip_request = TripRequestBuilder::new(&points)
        .build()
        .expect("Failed to build trip request");

    let trip_response = engine.trip(trip_request).expect("Failed navigate trip");

    assert_eq!(trip_response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        trip_response
            .trips
            .iter()
            .map(|r| r.legs.len())
            .sum::<usize>(),
        points.len(),
        "Route response length is incorrect"
    );
}

#[test]
fn test_native_route_steps() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
    ];
    let route_request = RouteRequestBuilder::new(&points)
        .steps(true)
        .build()
        .expect("No points in request");
    let response = engine
        .route(&route_request)
        .expect("Failed to route request");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    let n_legs = response.routes.iter().map(|r| r.legs.len()).sum::<usize>();
    assert_eq!(
        n_legs,
        points.len() - 1,
        "Route response length is incorrect"
    );
    let n_steps = response
        .routes
        .iter()
        .map(|r| r.legs.iter().map(|l| l.steps.len()).sum::<usize>())
        .sum::<usize>();
    assert_ne!(n_steps, 0, "Num steps should be greater than zero")
}

#[test]
fn test_native_route_annotations() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
    ];
    let route_request = RouteRequestBuilder::new(&points)
        .annotations(true)
        .build()
        .expect("No points in request");

    let response = engine
        .route(&route_request)
        .expect("Failed to route request");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    let n_legs = response.routes.iter().map(|r| r.legs.len()).sum::<usize>();
    assert_eq!(
        n_legs,
        points.len() - 1,
        "Route response length is incorrect"
    );
    assert!(
        response
            .routes
            .first()
            .unwrap()
            .legs
            .first()
            .unwrap()
            .annotation
            .is_some(),
        "Response should have annotations"
    )
}

#[test]
fn test_native_route_geometries() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
    ];
    let route_request = RouteRequestBuilder::new(&points)
        .geometry(GeometryType::GeoJSON)
        .overview(OverviewZoom::Full)
        .build()
        .expect("No points in request");

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

    let route_request = RouteRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline6)
        .overview(OverviewZoom::Full)
        .build()
        .expect("No points in request");

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
fn test_native_nearest() {
    let engine = init_native_engine(".env");

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
fn test_native_table() {
    let engine = init_native_engine(".env");

    let sources = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.040437, 10.216550).expect("Invalid point"),
    ];
    let destinations = [
        Point::new(48.540437, 10.816550).expect("Invalid point"),
        Point::new(49.140437, 10.416550).expect("Invalid point"),
        Point::new(49.140437, 10.516550).expect("Invalid point"),
    ];
    let table_request = TableRequestBuilder::new(&sources, &destinations)
        .build()
        .expect("Failed to create table request");
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
}
#[test]
fn test_table_options() {
    let engine = init_native_engine(".env");

    let sources = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.040437, 10.216550).expect("Invalid point"),
    ];
    let destinations = [
        Point::new(48.540437, 10.816550).expect("Invalid point"),
        Point::new(49.140437, 10.416550).expect("Invalid point"),
        Point::new(49.140437, 10.516550).expect("Invalid point"),
    ];
    let table_request = TableRequestBuilder::new(&sources, &destinations)
        .annotations(osrm_interface::table::TableAnnotation::All)
        .build()
        .expect("Failed to create table request");
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

#[test]
fn test_match_basic() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(51.097683869065804, 11.517827906178626).expect("Invalid point"),
        Point::new(51.098737989249116, 11.526971690952534).expect("Invalid point"),
        Point::new(51.09937599770893, 11.530571780087442).expect("Invalid point"),
        Point::new(51.099195691869646, 11.535806265573953).expect("Invalid point"),
        Point::new(51.09883507808152, 11.541924208526543).expect("Invalid point"),
        Point::new(51.10019429998817, 11.547070348266445).expect("Invalid point"),
        Point::new(51.10187286817584, 11.549241921250635).expect("Invalid point"),
        Point::new(51.10307204569769, 11.561966320703071).expect("Invalid point"),
    ];

    let match_request = MatchRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::False)
        .gaps(osrm_interface::r#match::MatchGapsBehaviour::Ignore)
        .build()
        .expect("Failed to create match request");
    let response = engine
        .r#match(&match_request)
        .expect("Failed to match route");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
}

#[test]
fn test_match_bearings() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(51.79158510923947, 10.935907962485363).expect("Invalid point"),
        Point::new(51.79102893132051, 10.937499982724786).expect("Invalid point"),
        Point::new(51.78855795961325, 10.94134736497006).expect("Invalid point"),
        Point::new(51.78771907497081, 10.943263685628628).expect("Invalid point"),
        Point::new(51.787855850705256, 10.939548971736638).expect("Invalid point"),
        Point::new(51.78957918942483, 10.934315942245938).expect("Invalid point"),
        Point::new(51.7864151254788, 10.929436695030665).expect("Invalid point"),
        Point::new(51.78421747479142, 10.927785711307926).expect("Invalid point"),
    ];

    let bearings = [
        Bearing::new(180, 90),
        None,
        Bearing::new(180, 90),
        Bearing::new(180, 90),
        Bearing::new(180, 90),
        None,
        None,
        Bearing::new(180, 90),
    ];

    let match_request = MatchRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::False)
        .gaps(osrm_interface::r#match::MatchGapsBehaviour::Ignore)
        .bearings(&bearings)
        .build()
        .expect("Failed to create match request");
    let response = engine
        .r#match(&match_request)
        .expect("Failed to match route");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
}

#[test]
fn test_match_timestamps() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(51.097683869065804, 11.517827906178626).expect("Invalid point"),
        Point::new(51.098737989249116, 11.526971690952534).expect("Invalid point"),
        Point::new(51.09937599770893, 11.530571780087442).expect("Invalid point"),
        Point::new(51.099195691869646, 11.535806265573953).expect("Invalid point"),
        Point::new(51.09883507808152, 11.541924208526543).expect("Invalid point"),
        Point::new(51.10019429998817, 11.547070348266445).expect("Invalid point"),
        Point::new(51.10187286817584, 11.549241921250635).expect("Invalid point"),
        Point::new(51.10307204569769, 11.561966320703071).expect("Invalid point"),
    ];

    // Generate some timestamps for the points based on their straight line distance
    let start_time = 1_730_000_000;
    let speed_mps = 16.7; // ~60 km/h
    let timestamps = std::iter::once(start_time)
        .chain(
            points
                .windows(2)
                .map(|p| {
                    let dlat = (p[1].latitude() - p[0].latitude()) * PI / 180.0;
                    let dlon = (p[1].longitude() - p[0].longitude()) * PI / 180.0;
                    let mean_lat = (p[0].latitude() + p[1].latitude()) * 0.5 * PI / 180.0;
                    let distance =
                        6_371_000.0 * ((dlon * mean_lat.cos()).powi(2) + dlat.powi(2)).sqrt();
                    (distance / speed_mps).ceil() as u64
                })
                .scan(start_time, |t, dt| {
                    *t += dt;
                    Some(*t)
                }),
        )
        .collect::<Vec<_>>();

    let match_request = MatchRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::False)
        .gaps(osrm_interface::r#match::MatchGapsBehaviour::Split)
        .timestamps(&timestamps)
        .build()
        .expect("Failed to create match request");
    let response = engine
        .r#match(&match_request)
        .expect("Failed to match route");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
}
#[test]
fn test_route_exclude() {
    let engine = init_native_engine(".env");

    // // Very short example entirely within Dresden
    // let points = [
    //     Point::new(51.08460070137968, 13.693104319460645).expect("Invalid point"),
    //     Point::new(51.10033848278219, 13.715837111172739).expect("Invalid point"),
    // ];
    // // Longer example from Dresden to Meissen
    // let points = [
    //     Point::new(51.084772300037436, 13.692948885858327).expect("Invalid point"),
    //     Point::new(51.16327210863313, 13.481085200778807).expect("Invalid point"),
    // ];
    // Very long example from Kiel to Munich
    let points = [
        Point::new(54.32049240451713, 10.125531530445544).expect("Invalid point"),
        Point::new(48.139305460041165, 11.579353374415504).expect("Invalid point"),
    ];

    let route_request = RouteRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::Full)
        .build()
        .expect("Failed to create route request");
    let normal_response = engine.route(&route_request).expect("Failed to route");

    let route_request = RouteRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::Full)
        .exclude(&[Exclude::Car(CarExclude::Motorway)])
        .build()
        .expect("Failed to create route request");
    let exclude_response = engine.route(&route_request).expect("Failed to route");

    assert_eq!(normal_response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(exclude_response.code, "Ok", "Response code is not 'Ok'");
    assert_ne!(
        normal_response.routes[0].duration, exclude_response.routes[0].duration,
        "Motorway excluded route has same duration as motorway inclusive route"
    );

    println!(
        "{} {}",
        normal_response.routes[0].duration, exclude_response.routes[0].duration
    )
}

#[test]
fn test_route_skip_waypoints() {
    let engine = init_native_engine(".env");

    let points = [
        Point::new(51.08460070137968, 13.693104319460645).expect("Invalid point"),
        Point::new(51.10033848278219, 13.715837111172739).expect("Invalid point"),
    ];

    let route_request = RouteRequestBuilder::new(&points)
        .geometry(GeometryType::Polyline)
        .overview(OverviewZoom::False)
        .skip_waypoints(true)
        .build()
        .expect("Failed to create route request");
    let response = engine.route(&route_request).expect("Failed to route");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert!(
        response.waypoints.is_none(),
        "Waypoints were returned despite skip_waypoints=true"
    )
}
