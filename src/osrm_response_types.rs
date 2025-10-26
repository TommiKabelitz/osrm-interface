//! The various response types contained in the responses from OSRM services.
//!
//! Documentation is pulled directly from the osrm-backend documentation in
//! v6.0.0 where it exists.

/// Represents a route through (potentially multiple) waypoints.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Route {
    /// The distance traveled by the route, in meters.
    pub distance: f64,
    /// The estimated travel time, in number of seconds.
    pub duration: f64,
    /// The whole geometry of the route value depending on the `overview` parameter,
    /// format depending on the `geometries` parameter.
    ///
    /// | overview   | Description                                                                                   |
    /// |------------|-----------------------------------------------------------------------------------------------|
    /// | simplified | Geometry is simplified according to the highest zoom level it can still be displayed in full. |
    /// | full       | Geometry is not simplified.                                                                   |
    /// | false      | Geometry is not added.                                                                        |
    pub geometry: Option<Geometry>,
    /// The calculated weight of the route.
    pub weight: f64,
    /// The name of the weight profile used during the extraction phase.
    pub weight_name: String,
    /// The legs between the given waypoints, an array of `RouteLeg` objects.
    pub legs: Vec<RouteLeg>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            distance: 90.0,
            duration: 300.0,
            weight: 300.0,
            weight_name: "duration".to_string(),
            geometry: Some(Geometry::GeoJson(GeoJsonLineString::default())),
            legs: vec![RouteLeg::default(), RouteLeg::default()],
        }
    }
}

/// Represents the geometry of a route or route step, either as a compact
/// polyline string or as a structured GeoJSON LineString.
#[cfg_attr(feature = "debug", derive(Debug))]
// #[cfg_attr(
//     any(feature = "native", feature = "remote"),
//     derive(serde::Deserialize)
// )]
// #[cfg_attr(any(feature = "native", feature = "remote"), serde(untagged))]
pub enum Geometry {
    /// Encoded polyline string (precision 5 or 6 depending on request)
    Polyline(String),
    /// GeoJSON LineString representation
    GeoJson(GeoJsonLineString),
}

// The approach of this implementation may need to change when support
// for flatbuffers is added
#[cfg(any(feature = "native", feature = "remote"))]
impl<'de> serde::Deserialize<'de> for Geometry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = <&serde_json::value::RawValue>::deserialize(deserializer)?;
        let trimmed = raw.get().trim_start();

        if trimmed.starts_with('{') {
            Ok(Geometry::GeoJson(
                serde_json::from_str(trimmed).map_err(|e|
                    match e.classify() {
                        serde_json::error::Category::Syntax => {
                            let index = crate::get_index_of_line_col(trimmed, e.line(), e.column()).unwrap();
                            let min = 0.max(index - 20);
                            let max = trimmed.len().min( index + 20);
                            serde::de::Error::custom(format!("Syntax error when attempting to parse geojson at line {}, col {}. Error occurred in this substring {}", e.line(), e.column(), trimmed.get(min..max).unwrap()))
                        }
                        serde_json::error::Category::Eof => {
                            let len = trimmed.len();
                            serde::de::Error::custom(format!("Reached end of file while parsing GeoJson. Start: {}, End: {}",
                            trimmed.get(..10.min(len)).unwrap(),
                            trimmed.get(len.saturating_sub(10)..).unwrap()
                        ))
                        }
                        e => serde::de::Error::custom(format!("{:?}",e)),
                    }
                )?),
            )
        } else if trimmed.starts_with('\"') {
            Ok(Geometry::Polyline(serde_json::from_str(trimmed).map_err(
                |e| match e.classify() {
                    serde_json::error::Category::Eof => {
                        let len = trimmed.len();
                        serde::de::Error::custom(format!(
                            "Reached end of file while parsing GeoJson. Start: {}, End: {}",
                            trimmed.get(..10.min(len)).unwrap(),
                            trimmed.get(len.saturating_sub(10)..).unwrap()
                        ))
                    }
                    e => serde::de::Error::custom(format!("{:?}", e)),
                },
            )?))
        // Want to ensure we can deserialize, even if we have json inside a string with the " in
        // the json escaped. In that case, the string will start with a \". It could have whitespace
        // after that, but just assuming it doesn't right now
        } else if trimmed.starts_with("\"{") {
            let inner: String = serde_json::from_str(trimmed).map_err(serde::de::Error::custom)?;

            // Then parse the unescaped JSON
            Ok(Geometry::GeoJson(
                serde_json::from_str(&inner).map_err(serde::de::Error::custom)?,
            ))
        } else {
            Err(serde::de::Error::custom(
                "Failed to parse geometry as Polyline or GeoJson",
            ))
        }
    }
}

/// GeoJSON LineString object for route geometry
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct GeoJsonLineString {
    /// Always `"LineString"`
    pub r#type: String,
    /// List of `[longitude, latitude]` coordinates
    pub coordinates: Vec<[f64; 2]>,
}
impl Default for GeoJsonLineString {
    fn default() -> Self {
        Self {
            r#type: "LineString".to_string(),
            coordinates: vec![[120.0, 10.0], [120.1, 10.0], [120.2, 10.0], [120.3, 10.0]],
        }
    }
}

/// Represents a route between two waypoints.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct RouteLeg {
    /// The distance traveled by this route leg, in meters.
    pub distance: f64,
    /// The estimated travel time, in number of seconds.
    pub duration: f64,
    /// The calculated weight of the route leg.
    pub weight: f64,
    /// Summary of the route taken as a string. Depends on the `steps` parameter:
    ///
    /// | steps |                                                                       |
    /// |-------|-----------------------------------------------------------------------|
    /// | true  | Names of the two major roads used. Can be empty if the route is too short. |
    /// | false | empty string                                                          |
    pub summary: String,
    /// Depends on the `steps` parameter:
    ///
    /// | steps |                                                                       |
    /// |-------|-----------------------------------------------------------------------|
    /// | true  | Array of `RouteStep` objects describing the turn-by-turn instructions |
    /// | false | Empty array                                                           |
    pub steps: Vec<RouteStep>,
    /// Additional details about each coordinate along with the route geometry.
    ///
    /// | annotations |                                                                 |
    /// |-------------|-----------------------------------------------------------------|
    /// | true        | An `Annotation` object containing node ids, durations, distances, and weights |
    /// | false       | `None`                                                          |
    pub annotation: Option<Annotation>,
}

impl Default for RouteLeg {
    fn default() -> Self {
        Self {
            distance: 30.0,
            duration: 100.0,
            weight: 100.0,
            summary: "".to_string(),
            steps: vec![], // empty because steps=false in example
            annotation: Some(Annotation::default()),
        }
    }
}

/// Annotation of the whole route leg with fine-grained information about each
/// segment or node id.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Annotation {
    /// The distance, in meters, between each pair of coordinates.
    pub distance: Vec<f64>,
    /// The duration, in seconds, between each pair of coordinates.
    /// Does not include the duration of any turns.
    pub duration: Vec<f64>,
    /// The index of the data source for the speed between each pair of coordinates.
    /// `0` is the default profile, other values are supplied via `--segment-speed-file`
    /// to `osrm-contract` or `osrm-customize`. String-like names are in the
    /// `metadata.datasource_names` array.
    pub datasources: Vec<u64>,
    /// The OSM node ID for each coordinate along the route, excluding the
    /// first/last user-supplied coordinates.
    pub nodes: Vec<f64>,
    /// The weights between each pair of coordinates.
    /// Does not include any turn costs.
    pub weight: Vec<f64>,
    /// Convenience field: calculation of `distance / duration` rounded to one decimal place.
    pub speed: Vec<f64>,
    /// Metadata related to other annotations.
    pub metadata: Metadata,
}

impl Default for Annotation {
    fn default() -> Self {
        Self {
            distance: vec![5.0, 5.0, 10.0, 5.0, 5.0],
            duration: vec![15.0, 15.0, 40.0, 15.0, 15.0],
            datasources: vec![1, 0, 0, 0, 1],
            nodes: vec![
                49772551.0, 49772552.0, 49786799.0, 49786800.0, 49786801.0, 49786802.0,
            ],
            weight: vec![15.0, 15.0, 40.0, 15.0, 15.0],
            speed: vec![], // not present in example, so empty
            metadata: Metadata::default(),
        }
    }
}

/// A step consists of a maneuver such as a turn or merge,
/// followed by a distance of travel along a single way to the subsequent step.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct RouteStep {
    /// The distance of travel from the maneuver to the subsequent step, in meters.
    pub distance: f64,
    /// The estimated travel time, in number of seconds.
    pub duration: f64,
    /// The calculated weight of the step.
    pub weight: f64,
    /// The name of the way along which travel proceeds.
    pub name: String,
    /// A reference number or code for the way.
    /// Optionally included, if reference data is available for the given way.
    pub r#ref: Option<String>,
    /// A string containing an IPA phonetic transcription indicating how to
    /// pronounce the `name`. Omitted if pronunciation data is unavailable.
    pub pronunciation: Option<String>,
    /// The destinations of the way. Will be `None` if there are no destinations.
    pub destinations: Option<String>,
    /// The exit numbers or names of the way. Will be `None` if there are no exits.
    pub exits: Option<String>,
    /// The name for the rotary. Optionally included if the step is a rotary
    /// and a rotary name is available.
    pub rotary_name: Option<String>,
    /// The pronunciation hint of the rotary name. Optionally included if the
    /// step is a rotary and a rotary pronunciation is available.
    pub rotary_pronunciation: Option<String>,
    /// A string signifying the mode of transportation.
    pub mode: DrivingMode,
    /// A `StepManeuver` object representing the maneuver.
    pub maneuver: StepManeuver,
    /// The unsimplified geometry of the route segment, depending on the
    /// `geometries` parameter.
    ///
    /// | geometry  | Description                                                                 |
    /// |-----------|-----------------------------------------------------------------------------|
    /// | polyline  | Encoded polyline with precision 5 in `[latitude,longitude]` order           |
    /// | polyline6 | Encoded polyline with precision 6 in `[latitude,longitude]` order           |
    /// | geojson   | GeoJSON `LineString`                                                        |
    pub geometry: Geometry,
    /// The legal driving side at the location for this step. Either `left` or `right`.
    pub driving_side: DrivingSide,
    /// A list of `Intersection` objects passed along the segment,
    /// the very first belonging to the `StepManeuver`.
    pub intersections: Vec<Intersection>,
}

impl Default for RouteStep {
    fn default() -> Self {
        Self {
            geometry: Geometry::Polyline("{lu_IypwpAVrAvAdI".to_string()),
            mode: DrivingMode::Driving,
            duration: 15.6,
            weight: 15.6,
            intersections: vec![Intersection::default(), Intersection::default()],
            name: "Lortzingstraße".to_string(),
            distance: 152.3,
            maneuver: StepManeuver::default(),
            r#ref: None,
            pronunciation: None,
            destinations: None,
            exits: None,
            rotary_name: None,
            rotary_pronunciation: None,
            driving_side: DrivingSide::Right,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    serde(rename_all = "lowercase")
)]
pub enum DrivingSide {
    Right,
    Left,
}

/// Additional metadata related to annotations (used by `Annotation`).
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Metadata {
    /// The names of the data sources used for the speeds between each coordinate segment.
    /// For example, “lua profile” for default, or names from supplied `--segment-speed-file`s.
    pub datasource_names: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            datasource_names: vec!["lua profile".to_string()],
        }
    }
}

/// An intersection gives a full representation of any cross-way the path passes by.
/// For every step, the very first intersection (`intersections[0]`) corresponds
/// to the location of the `StepManeuver`. Further intersections are listed for every
/// cross-way until the next turn instruction.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Intersection {
    /// A [longitude, latitude] pair describing the location of the intersection.
    pub location: [f64; 2],
    /// A list of bearing values (0-359) that are available at the intersection.
    /// These describe all available roads at the intersection.
    pub bearings: Vec<u16>,
    /// Classes of the roads exiting the intersection (as specified in the routing profile).
    pub classes: Option<Vec<String>>,
    /// List of entry flags, corresponding 1:1 with `bearings`.
    /// `true` indicates the road can be entered on a valid route,
    /// `false` indicates a restriction.
    pub entry: Vec<bool>,
    /// Index into bearings/entry array for the incoming road.
    /// Used to calculate the bearing just before the turn. Not supplied for `depart`.
    pub r#in: Option<usize>,
    /// Index into bearings/entry array for the outgoing road.
    /// Used to extract the bearing just after the turn. Not supplied for `arrive`.
    pub out: Option<usize>,
    /// Array of `Lane` objects that denote the available turn lanes at the intersection.
    /// If no lane information is available, this is `None`.
    pub lanes: Option<Vec<Lane>>,
}

impl Default for Intersection {
    fn default() -> Self {
        Self {
            location: [13.394718, 52.543096],
            bearings: vec![60, 150, 240, 330],
            classes: Some(vec!["toll".to_string(), "restricted".to_string()]),
            entry: vec![false, true, true, true],
            r#in: Some(0),
            out: Some(2),
            lanes: Some(vec![Lane::default()]),
        }
    }
}

/// The object is used to describe the waypoint on a route.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Waypoint {
    /// Unique internal identifier of the segment (ephemeral, not constant over data
    /// updates) This can be used on subsequent requests to significantly speed up the
    /// query and to connect multiple services. E.g. you can use the `hint` value
    /// obtained by the `nearest` query as `hint` values for `route` inputs.
    pub hint: String,
    /// Array that contains the [longitude, latitude] pair of the snapped coordinate
    pub location: [f64; 2],
    /// Name of the street the coordinate snapped to
    pub name: String,
    /// The distance, in meters, from the input coordinate to the snapped coordinate
    pub distance: f64,
}

impl Default for Waypoint {
    fn default() -> Self {
        Self {
            hint:
                "KSoKADRYroqUBAEAEAAAABkAAAAGAAAAAAAAABhnCQCLtwAA_0vMAKlYIQM8TMwArVghAwEAAQH1a66g"
                    .to_string(),
            location: [13.388799, 52.517033],
            distance: 4.152629,
            name: "Friedrichstraße".to_string(),
        }
    }
}

/// Represents a maneuver in a route step, such as a turn or merge.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct StepManeuver {
    /// A [longitude, latitude] pair describing the location of the maneuver.
    pub location: [f64; 2],
    /// The clockwise angle from true north to the direction of travel immediately before the maneuver.
    /// Range 0-359. Not supplied for `depart` maneuvers.
    pub bearing_before: f64,
    /// The clockwise angle from true north to the direction of travel immediately after the maneuver.
    /// Range 0-359. Not supplied for `arrive` maneuvers.
    pub bearing_after: f64,
    /// A string indicating the type of maneuver.
    ///
    /// Examples include: `turn`, `new name`, `depart`, `arrive`, `merge`,
    /// `on ramp`, `off ramp`, `fork`, `end of road`, `continue`, `roundabout`, `rotary`, etc.
    /// Types unknown to the client should be treated like `turn`.
    pub r#type: String,
    /// Optional string indicating the direction change of the maneuver.
    ///
    /// Examples: `uturn`, `sharp right`, `right`, `slight right`, `straight`,
    /// `slight left`, `left`, `sharp left`. Only `depart`/`arrive` may omit it.
    pub modifier: Option<String>,
    /// Optional integer indicating the exit number to take for `roundabout` / `rotary` maneuvers.
    pub exit: Option<u64>,
}

impl Default for StepManeuver {
    fn default() -> Self {
        Self {
            location: [13.39677, 52.54366],
            bearing_before: 0.0,
            bearing_after: 0.0,
            r#type: "turn".to_string(),
            modifier: Some("right".to_string()),
            exit: None,
        }
    }
}

/// A `Lane` represents a turn lane at the corresponding turn location.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct Lane {
    /// An array of indications (e.g., markings on the road) specifying the turn lane.
    /// Each indication may be one of:
    ///
    /// - `none`: No dedicated indication is shown.
    /// - `uturn`: Arrow indicating a reversal of direction.
    /// - `sharp right`: Arrow indicating a sharp right turn.
    /// - `right`: Arrow indicating a right turn.
    /// - `slight right`: Arrow indicating a slight right turn.
    /// - `straight`: Straight arrow (no dedicated indication).
    /// - `slight left`: Arrow indicating a slight left turn.
    /// - `left`: Arrow indicating a left turn.
    /// - `sharp left`: Arrow indicating a sharp left turn.
    pub indications: Vec<String>,
    /// A boolean flag indicating whether this lane is a valid choice for the current maneuver.
    pub valid: bool,
}

impl Default for Lane {
    fn default() -> Self {
        Self {
            indications: vec!["left".to_string(), "straight".to_string()],
            valid: false,
        }
    }
}

/// Represents the mode of transportation for a given `RouteStep`.
/// These values are pulled directly from OSRM’s source code.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub enum DrivingMode {
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "inaccessible")
    )]
    /// Segment cannot be accessed by the current profile
    Inaccessible,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "driving"))]
    /// Standard vehicular driving
    Driving,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "cycling"))]
    /// Cycling / bike mode
    Cycling,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "walking"))]
    /// Walking / pedestrian mode
    Walking,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "ferry"))]
    /// Travel by ferry
    Ferry,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "train"))]
    /// Travel by train
    Train,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "pushing bike")
    )]
    /// Walking while pushing a bicycle
    PushingBike,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "steps up")
    )]
    /// Going up steps (pedestrian)
    StepsUp,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "steps down")
    )]
    /// Going down steps (pedestrian)
    StepsDown,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "river upstream")
    )]
    /// Travel upstream on a river
    RiverUpstream,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "river downstream")
    )]
    /// Travel downstream on a river
    RiverDownstream,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "route"))]
    /// Generic or unspecified route segment
    Route,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "other", other)
    )]
    /// Fallback for unknown or custom segment types
    Other,
}

/// Represents a directional instruction used in both `StepManeuver.modifier`
/// and `Lane.indications`. Values describe the relative change in direction
/// or lane markings at an intersection.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub enum Direction {
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "uturn"))]
    Uturn,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "sharp right")
    )]
    SharpRight,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "right"))]
    Right,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "slight right")
    )]
    SlightRight,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "straight")
    )]
    Straight,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "slight left")
    )]
    SlightLeft,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "left"))]
    Left,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "sharp left")
    )]
    SharpLeft,
    #[cfg_attr(any(feature = "native", feature = "remote"), serde(rename = "none"))]
    /// Represents the absence of a directional marking (used in `Lane.indications`).
    None,
    #[cfg_attr(
        any(feature = "native", feature = "remote"),
        serde(rename = "other", other)
    )]
    /// Fallback for unknown or future direction strings not explicitly listed.
    Other,
}

/// The object is used to describe the waypoint on a route returned from the match
/// service.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct MatchWaypoint {
    /// Unique internal identifier of the segment (ephemeral, not constant over data
    /// updates) This can be used on subsequent requests to significantly speed up the
    /// query and to connect multiple services. E.g. you can use the `hint` value
    /// obtained by the `nearest` query as `hint` values for `route` inputs.
    pub hint: String,
    /// Array that contains the [longitude, latitude] pair of the snapped coordinate
    pub location: [f64; 2],
    /// Name of the street the coordinate snapped to
    pub name: String,
    /// The distance, in meters, from the input coordinate to the snapped coordinate
    pub distance: f64,
    /// Index to the `Route` object in `matchings` the sub-trace was matched to
    pub matchings_index: u64,
    /// Index of the waypoint inside the matched route
    pub waypoint_index: u64,
    /// Number of probable alternative matchings for this tracepoint. A value of zero indicates that this point was matched unambiguously. Split the trace at these points for incremental map matching
    pub alternatives_count: u64,
}

impl Default for MatchWaypoint {
    fn default() -> Self {
        Self {
            hint:
                "KSoKADRYroqUBAEAEAAAABkAAAAGAAAAAAAAABhnCQCLtwAA_0vMAKlYIQM8TMwArVghAwEAAQH1a66g"
                    .to_string(),
            location: [13.388799, 52.517033],
            distance: 4.152629,
            name: "Friedrichstraße".to_string(),
            matchings_index: 0,
            waypoint_index: 0,
            alternatives_count: 0,
        }
    }
}

/// Represents a route through (potentially multiple) waypoints.
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct MatchRoute {
    /// The distance traveled by the route, in meters.
    pub distance: f64,
    /// The estimated travel time, in number of seconds.
    pub duration: f64,
    /// The whole geometry of the route value depending on the `overview` parameter,
    /// format depending on the `geometries` parameter.
    ///
    /// | overview   | Description                                                                                   |
    /// |------------|-----------------------------------------------------------------------------------------------|
    /// | simplified | Geometry is simplified according to the highest zoom level it can still be displayed in full. |
    /// | full       | Geometry is not simplified.                                                                   |
    /// | false      | Geometry is not added.                                                                        |
    pub geometry: Option<Geometry>,
    /// The calculated weight of the route.
    pub weight: f64,
    /// The name of the weight profile used during the extraction phase.
    pub weight_name: String,
    /// The legs between the given waypoints, an array of `RouteLeg` objects.
    pub legs: Vec<RouteLeg>,
    /// Confidence of the matching. float value between 0 and 1. 1 is very confident that the matching is correct
    pub confidence: f64,
}

impl Default for MatchRoute {
    fn default() -> Self {
        Self {
            distance: 90.0,
            duration: 300.0,
            weight: 300.0,
            weight_name: "duration".to_string(),
            geometry: Some(Geometry::GeoJson(GeoJsonLineString::default())),
            legs: vec![RouteLeg::default(), RouteLeg::default()],
            confidence: 1.0,
        }
    }
}
