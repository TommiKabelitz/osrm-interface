//! The native version of the OSRM engine. Locked behind the `native`
//! feature flag. Calls directly into a linked version of osrm-backend
//! through a minimal cpp wrapper.
//!
//! ## Compilation
//!
//! Using this requires that osrm-backend is built on the system already.
//! Building and installing osrm-backend is likely to result in sufficient
//! headers being placed in the appropriate locations.
//!
//! For more details about compiling the backend, see the
//! [osrm documentation](<https://github.com/Project-OSRM/osrm-backend>).
//!
//! For compiling this crate, there are a couple of environment variables
//! you can leverage. The build.rs assumes that osrm-backend has been installed
//! to /usr/local/. If it is installed elsewhere, set `OSRM_BACKEND_PATH` to
//! the directory containing the backend (parent directory to the build dir).
//!
//! `OSRM_DEBUG_PATH` is also available which allows a different version of osrm
//! to be used for debug builds for things like debug symbols for stepping through
//! osrm itself in the debugger if you actually need it.
//!
//! Having said that, if you are having trouble, it may be easier to simply vendor
//! this crate into repo directly and then modifying the build.rs to point where
//! you need it to.
//!
//! ## Map data
//!
//! When initialising the engine, you will need to supply a path to the map data.
//! The map data is extracted using tools provided by the backend which are not
//! wrapped in this crate. See the
//! [osrm documentation](<https://github.com/Project-OSRM/osrm-backend>) for more
//! information about extracting the map data. The extraction process defines the
//! Algorithm that should be passed to `native::OsrmEngine::new()`.
//!

mod osrm_engine;
use crate::r#match::{Approach, MatchGapsBehaviour, MatchRequest};
use crate::nearest::NearestRequest;
use crate::request_types::{Bearing, Exclude, GeometryType, OverviewZoom, Snapping};
use crate::route::RouteRequest;
use crate::table::{TableAnnotation, TableFallbackCoordinate, TableRequest};
use crate::trip::{TripDestination, TripRequest, TripSource};
#[cfg_attr(doc, doc(cfg(feature = "native")))]
pub use osrm_engine::OsrmEngine;

use std::f64;
use std::ffi::{CStr, CString, c_void};
use std::os::raw::c_char;

const ROUTE_ALTERNATIVES: u8 = 1 << 0;
const ROUTE_STEPS: u8 = 1 << 1;
const ROUTE_ANNOTATIONS: u8 = 1 << 2;
const ROUTE_CONTINUE_STRAIGHT: u8 = 1 << 3;
const ROUTE_GENERATE_HINTS: u8 = 1 << 4;
const ROUTE_SKIP_WAYPOINTS: u8 = 1 << 5;

const MATCH_TIDY: u8 = 1 << 0;
const MATCH_STEPS: u8 = 1 << 1;
const MATCH_ANNOTATIONS: u8 = 1 << 2;
const MATCH_GENERATE_HINTS: u8 = 1 << 3;

const TRIP_STEPS: u8 = 1 << 0;
const TRIP_ANNOTATIONS: u8 = 1 << 1;
const TRIP_GENERATE_HINTS: u8 = 1 << 2;
const TRIP_SKIP_WAYPOINTS: u8 = 1 << 3;
const TRIP_ROUNDTRIP: u8 = 1 << 4;

#[repr(C)]
struct OsrmResult {
    code: i32,
    message: *mut c_char,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct ArrayString<'a> {
    len: usize,
    pointer: *const u8,
    _marker: std::marker::PhantomData<&'a u8>,
}

impl<'a> From<&'a str> for ArrayString<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            len: value.len(),
            pointer: value.as_ptr(),
            _marker: std::marker::PhantomData,
        }
    }
}

#[link(name = "osrm_wrapper", kind = "static")]
unsafe extern "C" {
    fn osrm_create(base_path: *const c_char, algorithm: *const c_char) -> *mut c_void;
    fn osrm_destroy(osrm_instance: *mut c_void);
    fn osrm_table(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        sources: *const usize,
        num_sources: usize,
        destinations: *const usize,
        num_destinations: usize,
        annotations: TableAnnotation,
        fallback_speed: f64,
        fallback_coordinate_type: TableFallbackCoordinate,
        scale_factor: f64,
        bearings: *const Bearing,
        num_bearings: usize,
        radiuses: *const f64,
        num_radiuses: usize,
        hints: *const ArrayString,
        num_hints: usize,
        approaches: *const Approach,
        num_approaches: usize,
        generate_hints: bool,
        excludes: *const ArrayString,
        num_excludes: usize,
        snapping: Snapping,
    ) -> OsrmResult;
    fn osrm_trip(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        geometry_type: GeometryType,
        overview_zoom: OverviewZoom,
        source: TripSource,
        destination: TripDestination,
        flags: u8,
        bearings: *const Bearing,
        num_bearings: usize,
        radiuses: *const f64,
        num_radiuses: usize,
        hints: *const ArrayString,
        num_hints: usize,
        approaches: *const Approach,
        num_approaches: usize,
        excludes: *const ArrayString,
        num_excludes: usize,
        snapping: Snapping,
    ) -> OsrmResult;

    fn osrm_route(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        geometry_type: GeometryType,
        overview_zoom: OverviewZoom,
        flags: u8,
        bearings: *const Bearing,
        num_bearings: usize,
        radiuses: *const f64,
        num_radiuses: usize,
        hints: *const ArrayString,
        num_hints: usize,
        approaches: *const Approach,
        num_approaches: usize,
        excludes: *const ArrayString,
        num_excludes: usize,
        snapping: Snapping,
    ) -> OsrmResult;

    fn osrm_match(
        osrm_instance: *mut c_void,
        coordinates: *const f64,
        num_coordinates: usize,
        geometry_type: GeometryType,
        overview_zoom: OverviewZoom,
        timestamps: *const u64,
        num_timestamps: usize,
        gaps_type: MatchGapsBehaviour,
        waypoints: *const usize,
        num_waypoints: usize,
        flags: u8,
        bearings: *const Bearing,
        num_bearings: usize,
        radiuses: *const f64,
        num_radiuses: usize,
        hints: *const ArrayString,
        num_hints: usize,
        approaches: *const Approach,
        num_approaches: usize,
        excludes: *const ArrayString,
        num_excludes: usize,
    ) -> OsrmResult;

    fn osrm_nearest(
        osrm_instance: *mut c_void,
        long: f64,
        lat: f64,
        number: u64,
        bearing: *const Bearing,
        radius: *const f64,
        approach: *const Approach,
        excludes: *const ArrayString,
        num_excludes: usize,
        snapping: *const Snapping,
    ) -> OsrmResult;

    fn osrm_last_error() -> *const c_char;
    fn osrm_free_string(s: *mut c_char);
}

pub(crate) struct Osrm {
    instance: *mut c_void,
}

impl Osrm {
    pub(crate) fn new(base_path: &str, algorithm: &str) -> Result<Self, String> {
        let c_path = CString::new(base_path).map_err(|e| e.to_string())?;
        let c_algorithm = CString::new(algorithm).map_err(|e| e.to_string())?;
        let instance = unsafe { osrm_create(c_path.as_ptr(), c_algorithm.as_ptr()) };

        if instance.is_null() {
            let err_ptr = unsafe { osrm_last_error() };
            let msg = if err_ptr.is_null() {
                "unknown error".to_string()
            } else {
                unsafe { CStr::from_ptr(err_ptr).to_string_lossy().into_owned() }
            };
            Err(format!("Failure to create an OSRM instance: {}", msg))
        } else {
            Ok(Osrm { instance })
        }
    }

    pub(crate) fn trip(&self, trip_request: &TripRequest) -> Result<String, String> {
        let num_coords = trip_request.points.len();
        let coords: Vec<f64> = trip_request
            .points
            .iter()
            .flat_map(|p| [p.longitude(), p.latitude()])
            .collect();

        let bearings = if let Some(bearings) = trip_request.bearings {
            bearings
                .iter()
                .map(|bearing| bearing.unwrap_or_default())
                .collect()
        } else {
            Vec::new()
        };

        let radiuses = match trip_request.radiuses {
            Some(rad) => rad.iter().map(|f| f.unwrap_or(f64::INFINITY)).collect(),
            None => vec![f64::INFINITY; num_coords],
        };
        let hints = match trip_request.hints {
            Some(hints) => hints.iter().map(|hint| hint.unwrap_or("").into()).collect(),
            None => Vec::new(),
        };
        let approaches = trip_request.approaches.unwrap_or(&[]);

        let excludes = match trip_request.exclude {
            Some(excludes) => excludes
                .iter()
                .map(|exclude| match exclude {
                    Exclude::Bicycle(v) => v.as_str().into(),
                    Exclude::Car(v) => v.as_str().into(),
                })
                .collect(),
            None => Vec::new(),
        };
        let snapping = trip_request.snapping.unwrap_or(Snapping::Default);
        let mut flags: u8 = 0;
        if trip_request.steps {
            flags |= TRIP_STEPS;
        }
        if trip_request.annotations {
            flags |= TRIP_ANNOTATIONS;
        }
        if trip_request.generate_hints {
            flags |= TRIP_GENERATE_HINTS;
        }
        if trip_request.skip_waypoints {
            flags |= TRIP_SKIP_WAYPOINTS;
        }
        if trip_request.roundtrip {
            flags |= TRIP_ROUNDTRIP;
        }

        let result = unsafe {
            osrm_trip(
                self.instance,
                coords.as_ptr(),
                num_coords,
                trip_request.geometry,
                trip_request.overview,
                trip_request.source,
                trip_request.destination,
                flags,
                bearings.as_ptr(),
                bearings.len(),
                radiuses.as_ptr(),
                radiuses.len(),
                hints.as_ptr(),
                hints.len(),
                approaches.as_ptr(),
                approaches.len(),
                excludes.as_ptr(),
                excludes.len(),
                snapping,
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn route(&self, route_request: &RouteRequest) -> Result<String, String> {
        let num_coords = route_request.points.len();
        let coords: Vec<f64> = route_request
            .points
            .iter()
            .flat_map(|p| [p.longitude(), p.latitude()])
            .collect();
        let bearings = if let Some(bearings) = route_request.bearings {
            bearings
                .iter()
                .map(|bearing| bearing.unwrap_or_default())
                .collect()
        } else {
            Vec::new()
        };

        let radiuses = match route_request.radiuses {
            Some(rad) => rad.iter().map(|f| f.unwrap_or(f64::INFINITY)).collect(),
            None => vec![f64::INFINITY; num_coords],
        };
        let hints = match route_request.hints {
            Some(hints) => hints.iter().map(|hint| hint.unwrap_or("").into()).collect(),
            None => Vec::new(),
        };
        let approaches = route_request.approaches.unwrap_or(&[]);

        let excludes = match route_request.exclude {
            Some(excludes) => excludes
                .iter()
                .map(|exclude| match exclude {
                    Exclude::Bicycle(v) => v.as_str().into(),
                    Exclude::Car(v) => v.as_str().into(),
                })
                .collect(),
            None => Vec::new(),
        };
        let snapping = route_request.snapping.unwrap_or(Snapping::Default);
        let mut flags: u8 = 0;
        if route_request.alternatives {
            flags |= ROUTE_ALTERNATIVES;
        }
        if route_request.steps {
            flags |= ROUTE_STEPS;
        }
        if route_request.annotations {
            flags |= ROUTE_ANNOTATIONS;
        }
        if route_request.continue_straight {
            flags |= ROUTE_CONTINUE_STRAIGHT;
        }
        if route_request.generate_hints {
            flags |= ROUTE_GENERATE_HINTS;
        }
        if route_request.skip_waypoints {
            flags |= ROUTE_SKIP_WAYPOINTS;
        }
        let result = unsafe {
            osrm_route(
                self.instance,
                coords.as_ptr(),
                num_coords,
                route_request.geometry,
                route_request.overview,
                flags,
                bearings.as_ptr(),
                bearings.len(),
                radiuses.as_ptr(),
                radiuses.len(),
                hints.as_ptr(),
                hints.len(),
                approaches.as_ptr(),
                approaches.len(),
                excludes.as_ptr(),
                excludes.len(),
                snapping,
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn r#match(&self, match_request: &MatchRequest) -> Result<String, String> {
        let num_coords = match_request.points.len();
        let coords: Vec<f64> = match_request
            .points
            .iter()
            .flat_map(|p| [p.longitude(), p.latitude()])
            .collect();

        let mut flags: u8 = 0;
        if match_request.tidy {
            flags |= MATCH_TIDY;
        }
        if match_request.steps {
            flags |= MATCH_STEPS;
        }
        if match_request.annotations {
            flags |= MATCH_ANNOTATIONS;
        }
        if match_request.generate_hints {
            flags |= MATCH_GENERATE_HINTS
        }

        let timestamps = match_request.timestamps.unwrap_or(&[]);
        let waypoints = match_request.waypoints.unwrap_or(&[]);

        let bearings = if let Some(bearings) = match_request.bearings {
            bearings
                .iter()
                .map(|bearing| bearing.unwrap_or_default())
                .collect()
        } else {
            Vec::new()
        };

        let radiuses = match match_request.radiuses {
            Some(rad) => rad.iter().map(|f| f.unwrap_or(f64::INFINITY)).collect(),
            None => vec![f64::INFINITY; num_coords],
        };
        let hints = match match_request.hints {
            Some(hints) => hints.iter().map(|hint| hint.unwrap_or("").into()).collect(),
            None => Vec::new(),
        };
        let approaches = match_request.approaches.unwrap_or(&[]);
        let excludes = match match_request.exclude {
            Some(excludes) => excludes
                .iter()
                .map(|exclude| match exclude {
                    Exclude::Bicycle(v) => v.as_str().into(),
                    Exclude::Car(v) => v.as_str().into(),
                })
                .collect(),
            None => Vec::new(),
        };

        let result = unsafe {
            osrm_match(
                self.instance,
                coords.as_ptr(),
                num_coords,
                match_request.geometry,
                match_request.overview,
                timestamps.as_ptr(),
                timestamps.len(),
                match_request.gaps,
                waypoints.as_ptr(),
                waypoints.len(),
                flags,
                bearings.as_ptr(),
                bearings.len(),
                radiuses.as_ptr(),
                radiuses.len(),
                hints.as_ptr(),
                hints.len(),
                approaches.as_ptr(),
                approaches.len(),
                excludes.as_ptr(),
                excludes.len(),
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn table(&self, table_request: &TableRequest) -> Result<String, String> {
        // Not using is_empty because the lengths are actually needed for the index
        // arrays below
        let len_sources = table_request.sources.len();
        let len_destinations = table_request.destinations.len();

        let sources_index: &[usize] = &(0..(len_sources)).collect::<Vec<usize>>()[..];
        let destination_index: &[usize] =
            &(len_sources..(len_sources + len_destinations)).collect::<Vec<usize>>()[..];
        let coordinates: &[(f64, f64)] = &[table_request.sources, table_request.destinations]
            .concat()
            .iter()
            .map(|s| (s.longitude(), s.latitude()))
            .collect::<Vec<(f64, f64)>>()[..];

        let flat_coords: Vec<f64> = coordinates
            .iter()
            .flat_map(|&(lon, lat)| vec![lon, lat])
            .collect();

        let bearings = if table_request.source_bearings.is_some()
            | table_request.destination_bearings.is_some()
        {
            let mut bearings = vec![Bearing::default(); len_sources + len_destinations];
            if let Some(source_bearings) = table_request.source_bearings {
                for (i, b) in source_bearings.iter().enumerate() {
                    if let Some(b) = b {
                        bearings[i] = *b;
                    }
                }
            }
            if let Some(destination_bearings) = table_request.destination_bearings {
                for (i, b) in destination_bearings.iter().enumerate() {
                    if let Some(b) = b {
                        bearings[len_sources + i] = *b;
                    }
                }
            }
            bearings
        } else {
            Vec::new()
        };

        let radiuses = if table_request.source_radiuses.is_some()
            | table_request.destination_radiuses.is_some()
        {
            let mut radiuses = vec![0.0; len_sources + len_destinations];
            if let Some(source_radiuses) = table_request.source_radiuses {
                for (i, r) in source_radiuses.iter().enumerate() {
                    if let Some(r) = r {
                        radiuses[i] = *r;
                    }
                }
            }
            if let Some(destination_radiuses) = table_request.destination_radiuses {
                for (i, r) in destination_radiuses.iter().enumerate() {
                    if let Some(r) = r {
                        radiuses[len_sources + i] = *r;
                    }
                }
            }
            radiuses
        } else {
            Vec::new()
        };

        let hints =
            if table_request.source_hints.is_some() | table_request.destination_hints.is_some() {
                let mut hints = vec![ArrayString::from(""); len_sources + len_destinations];
                if let Some(source_hints) = table_request.source_hints {
                    for (i, h) in source_hints.iter().enumerate() {
                        if let Some(h) = h {
                            hints[i] = ArrayString::from(*h);
                        }
                    }
                }
                if let Some(destination_hints) = table_request.destination_hints {
                    for (i, h) in destination_hints.iter().enumerate() {
                        if let Some(h) = h {
                            hints[len_sources + i] = ArrayString::from(*h);
                        }
                    }
                }
                hints
            } else {
                Vec::new()
            };

        let approaches = if table_request.source_approaches.is_some()
            | table_request.destination_approaches.is_some()
        {
            let mut approaches = vec![Approach::Unrestricted; len_sources + len_destinations];
            if let Some(source_approaches) = table_request.source_approaches {
                for (i, a) in source_approaches.iter().enumerate() {
                    approaches[i] = *a
                }
            }
            if let Some(destination_approaches) = table_request.destination_approaches {
                for (i, a) in destination_approaches.iter().enumerate() {
                    approaches[len_sources + i] = *a;
                }
            }
            approaches
        } else {
            Vec::new()
        };

        let excludes = match table_request.exclude {
            Some(excludes) => excludes
                .iter()
                .map(|exclude| match exclude {
                    Exclude::Bicycle(v) => v.as_str().into(),
                    Exclude::Car(v) => v.as_str().into(),
                })
                .collect(),
            None => Vec::new(),
        };
        let snapping = table_request.snapping.unwrap_or(Snapping::Default);

        let result = unsafe {
            osrm_table(
                self.instance,
                flat_coords.as_ptr(),
                coordinates.len(),
                sources_index.as_ptr(),
                sources_index.len(),
                destination_index.as_ptr(),
                destination_index.len(),
                table_request.annotations,
                table_request.fallback_speed.unwrap_or(0.0),
                table_request
                    .fallback_coordinate
                    .unwrap_or(TableFallbackCoordinate::Input),
                table_request.scale_factor.unwrap_or(0.0),
                bearings.as_ptr(),
                bearings.len(),
                radiuses.as_ptr(),
                radiuses.len(),
                hints.as_ptr(),
                hints.len(),
                approaches.as_ptr(),
                approaches.len(),
                table_request.generate_hints,
                excludes.as_ptr(),
                excludes.len(),
                snapping,
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }

    pub(crate) fn nearest(&self, nearest_request: &NearestRequest) -> Result<String, String> {
        let excludes = match nearest_request.exclude {
            Some(excludes) => excludes
                .iter()
                .map(|exclude| match exclude {
                    Exclude::Bicycle(v) => v.as_str().into(),
                    Exclude::Car(v) => v.as_str().into(),
                })
                .collect(),
            None => Vec::new(),
        };

        let result = unsafe {
            osrm_nearest(
                self.instance,
                nearest_request.point.longitude(),
                nearest_request.point.latitude(),
                nearest_request.number,
                nearest_request
                    .bearing
                    .as_ref()
                    .map_or(std::ptr::null(), |b| b as *const _),
                nearest_request
                    .radius
                    .as_ref()
                    .map_or(std::ptr::null(), |r| r as *const _),
                nearest_request
                    .approach
                    .as_ref()
                    .map_or(std::ptr::null(), |a| a as *const _),
                excludes.as_ptr(),
                excludes.len(),
                nearest_request
                    .snapping
                    .as_ref()
                    .map_or(std::ptr::null(), |s| s as *const _),
            )
        };

        let message_ptr = result.message;
        if message_ptr.is_null() {
            return Err("OSRM returned a null message".to_string());
        }

        let c_str = unsafe { CStr::from_ptr(message_ptr) };
        let rust_str = c_str.to_str().map_err(|e| e.to_string())?.to_owned();

        unsafe {
            osrm_free_string(message_ptr);
        }

        if result.code != 0 {
            return Err(format!("OSRM error: {}", rust_str));
        }

        Ok(rust_str)
    }
}

impl Drop for Osrm {
    fn drop(&mut self) {
        unsafe {
            osrm_destroy(self.instance);
        }
    }
}

unsafe impl Send for Osrm {}
unsafe impl Sync for Osrm {}
