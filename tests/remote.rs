#![cfg(feature = "remote")]

mod common;
use common::init_remote_engine;

use osrm_interface::{
    Point,
    r#match::MatchRequestBuilder,
    nearest::NearestRequestBuilder,
    osrm_response_types::Geometry,
    request_types::{CarExclude, Exclude, GeometryType, OverviewZoom},
    route::RouteRequestBuilder,
    table::TableRequestBuilder,
    trip::TripRequestBuilder,
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
    let route_request = RouteRequestBuilder::new(&points)
        .geometry(osrm_interface::request_types::GeometryType::GeoJSON)
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
fn test_basic_remote_trip() {
    let engine = init_remote_engine(".env");

    let points = [
        Point::new(48.040437, 10.316550).expect("Invalid point"),
        Point::new(49.006101, 9.052887).expect("Invalid point"),
        Point::new(48.942296, 10.510960).expect("Invalid point"),
        Point::new(51.248931, 7.594814).expect("Invalid point"),
    ];
    let trip_request = TripRequestBuilder::new(&points)
        .geometry(osrm_interface::request_types::GeometryType::GeoJSON)
        .build()
        .expect("Failed to build trip request");

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
fn test_remote_nearest() {
    let engine = init_remote_engine(".env");

    let num_points = 3;
    let point = Point::new(48.040437, 10.316550).expect("Invalid point");
    let nearest_request = NearestRequestBuilder::new(&point, num_points)
        .build()
        .expect("Failed to build nearest request");
    let response = engine
        .nearest(&nearest_request)
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
    let table_request = TableRequestBuilder::new(&sources, &destinations)
        .build()
        .expect("Failed to create table request");
    let response = engine
        .table(table_request)
        .expect("Failed to determine table");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.sources.unwrap().len(),
        sources.len(),
        "Returned more sources than anticipated"
    );
    assert_eq!(
        response.destinations.unwrap().len(),
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
    let table_request = TableRequestBuilder::new(&sources, &destinations)
        .annotations(osrm_interface::table::TableAnnotation::All)
        .build()
        .expect("Failed to create table request");
    let response = engine
        .table(table_request)
        .expect("Failed to determine table");

    assert_eq!(response.code, "Ok", "Response code is not 'Ok'");
    assert_eq!(
        response.sources.unwrap().len(),
        sources.len(),
        "Returned more sources than anticipated"
    );
    assert_eq!(
        response.destinations.unwrap().len(),
        destinations.len(),
        "Returned more destinations than anticipated"
    );
    assert!(response.distances.is_some(), "Distances should be Some");
    assert!(response.durations.is_some(), "Durations should be Some");
}

#[test]
fn test_match_basic() {
    let engine = init_remote_engine(".env");

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
fn test_route_exclude() {
    let engine = init_remote_engine(".env");

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
    )
}

#[test]
fn test_route_skip_waypoints() {
    let engine = init_remote_engine(".env");

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
