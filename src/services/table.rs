//! Given a set of source and destination coordinates, determine the
//! distance and/or duration to travel between those locations.

use thiserror::Error;

use crate::Point;
use crate::request_types::{Bearing, Exclude, Snapping};
use crate::services::{Approach, DimensionMismatch};

/// The request object passed to the table service. Constructed
/// through [`TableRequestBuilder::build`] which verifies the
/// validity of the request.
///
/// See [`TableRequestBuilder`] for more information on table requests.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TableRequest<'a> {
    pub(crate) sources: &'a [Point],
    pub(crate) destinations: &'a [Point],
    pub(crate) annotations: TableAnnotation,
    pub(crate) fallback_speed: Option<f64>,
    pub(crate) fallback_coordinate: Option<TableFallbackCoordinate>,
    pub(crate) scale_factor: Option<f64>,
    pub(crate) source_bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) destination_bearings: Option<&'a [Option<Bearing>]>,
    pub(crate) source_radiuses: Option<&'a [Option<f64>]>,
    pub(crate) destination_radiuses: Option<&'a [Option<f64>]>,
    pub(crate) generate_hints: bool,
    pub(crate) source_hints: Option<&'a [Option<&'a str>]>,
    pub(crate) destination_hints: Option<&'a [Option<&'a str>]>,
    pub(crate) source_approaches: Option<&'a [Approach]>,
    pub(crate) destination_approaches: Option<&'a [Approach]>,
    pub(crate) exclude: Option<&'a [Exclude]>,
    pub(crate) snapping: Option<Snapping>,
}

/// Helper struct for building a [`TableRequest`].
///
/// Set table options using the struct methods before calling
/// [`build`](Self::build). The `build` method validates the configuration
/// and attempts to detect invalid combinations before the request is sent
/// to the service.
///
/// ## Options
///
/// - **`sources`** (*required*) — A slice of [`Point`]s representing the source
///   coordinates. Must contain at least one point.
///
/// - **`destinations`** (*required*) — A slice of [`Point`]s representing the
///   destination coordinates. Must contain at least one point.
///
/// - **`annotations`** (*default:* `TableAnnotation::Duration`) — Specifies
///   which metrics to compute for each source-destination pair. See
///   [`TableAnnotation`] for available options.
///
/// - **`fallback`** (*optional pair: `coord`, `speed`*) — Specifies how to
///   handle un-routable source-destination pairs. The `coord` determines
///   the fallback coordinate type, and the `speed` gives the assumed travel
///   speed in meters per second. Both must be set together.
///
/// - **`scale_factor`** (*optional*) — Scales all returned durations by a
///   given factor. Only valid when duration annotations are requested.
///
/// - **`exclude`** (*optional*) — A slice of [`Exclude`] values, all of the same
///   transport mode (e.g., all `Exclude::Car` or all `Exclude::Bicycle`),
///   specifying road classes to exclude from the matrix computation.
///
/// - **`snapping`** (*optional*) — Defines how coordinates are snapped to
///   the road network. See [`Snapping`] for available modes.
///
/// - **`generate_hints`** (*default:* `true`) — When enabled, OSRM returns
///   location hints to accelerate subsequent queries.
///
/// ## Array options
///
/// The following options accept arrays that must correspond in length to
/// either the `sources` or `destinations` arrays, depending on their purpose.
/// Each array may contain [`Option`] elements to selectively override defaults.
///
/// - **`source_bearings`** (*optional*) — A slice of optional [`Bearing`]s, one
///   per source point. Each defines an allowed direction in which the point may
///   be snapped to a node. None => Any direction.
///
/// - **`destination_bearings`** (*optional*) — Like `source_bearings`, but for
///   destination points.
///
/// - **`source_radiuses`** (*optional*) — A slice of optional radiuses (in meters)
///   constraining how far OSRM may search from each source. `None` means infinite.
///
/// - **`destination_radiuses`** (*optional*) — Like `source_radiuses`, but for
///   destinations.
///
/// - **`source_hints`** (*optional*) — A slice of optional precomputed location
///   hints for each source point, to speed up matching.
///
/// - **`destination_hints`** (*optional*) — Like `source_hints`, but for
///   destinations.
///
/// - **`source_approaches`** (*optional*) — A slice of [`Approach`] values specifying
///   the side of the road from which each source point should be accessed.
///   Defaults to unrestricted access.
///
/// - **`destination_approaches`** (*optional*) — Like `source_approaches`, but for
///   destination points.
///
/// ## Example
///
/// ```
/// use osrm_interface::table::{TableRequestBuilder, TableAnnotation};
///
/// let sources = [
///     Point::new(48.040437, 10.316550).expect("Invalid point"),
///     Point::new(49.006101, 9.052887).expect("Invalid point"),
/// ];
///
/// let destinations = [
///     Point::new(48.942296, 10.510960).expect("Invalid point"),
///     Point::new(49.015000, 9.060000).expect("Invalid point"),
/// ];
///
/// let table_request = TableRequestBuilder::new(&sources, &destinations)
///     .annotations(TableAnnotation::DurationDistance)
///     .fallback(TableFallbackCoordinate::Input, 13.9)
///     .scale_factor(1.0)
///     .build()
///     .expect("Failed to build TableRequest");
/// ```
pub struct TableRequestBuilder<'a> {
    sources: &'a [Point],
    destinations: &'a [Point],
    annotations: TableAnnotation,
    fallback_speed: Option<f64>,
    fallback_coordinate: Option<TableFallbackCoordinate>,
    scale_factor: Option<f64>,
    source_bearings: Option<&'a [Option<Bearing>]>,
    destination_bearings: Option<&'a [Option<Bearing>]>,
    source_radiuses: Option<&'a [Option<f64>]>,
    destination_radiuses: Option<&'a [Option<f64>]>,
    generate_hints: bool,
    source_hints: Option<&'a [Option<&'a str>]>,
    destination_hints: Option<&'a [Option<&'a str>]>,
    source_approaches: Option<&'a [Approach]>,
    destination_approaches: Option<&'a [Approach]>,
    exclude: Option<&'a [Exclude]>,
    snapping: Option<Snapping>,
}

impl<'a> TableRequestBuilder<'a> {
    /// Creates a new [`TableRequestBuilder`] with the given sources and destinations.
    ///
    /// The builder can then be customized using its setter methods.
    pub fn new(sources: &'a [Point], destinations: &'a [Point]) -> Self {
        Self {
            sources,
            destinations,
            annotations: TableAnnotation::Duration,
            fallback_speed: None,
            fallback_coordinate: None,
            scale_factor: None,
            source_bearings: None,
            destination_bearings: None,
            source_radiuses: None,
            destination_radiuses: None,
            generate_hints: true,
            source_hints: None,
            destination_hints: None,
            source_approaches: None,
            destination_approaches: None,
            exclude: None,
            snapping: None,
        }
    }

    /// Overwrite the sources provided at construction of the builder. Useful
    /// for reusing a builder with the same options.
    ///
    /// Take care that array-like options for the sources are still the same
    /// length as sources, [`build`](Self::build) will throw an error if not.
    pub fn sources(mut self, sources: &'a [Point]) -> Self {
        self.sources = sources;
        self
    }

    /// Overwrite the destinations provided at construction of the builder. Useful
    /// for reusing a builder with the same options.
    ///
    /// Take care that array-like options for the destinations are still the same
    /// length as destinations, [`build`](Self::build) will throw an error if not.
    pub fn destinations(mut self, destinations: &'a [Point]) -> Self {
        self.destinations = destinations;
        self
    }

    /// Sets the requested table annotations (e.g., duration, distance).
    pub fn annotations(mut self, val: TableAnnotation) -> Self {
        self.annotations = val;
        self
    }

    /// Sets both the fallback coordinate type and fallback speed (in meters per second).
    ///
    /// Both parameters must be provided together.
    pub fn fallback(mut self, coord: TableFallbackCoordinate, speed: f64) -> Self {
        self.fallback_coordinate = Some(coord);
        self.fallback_speed = Some(speed);
        self
    }

    /// Sets the scale factor for adjusting table durations.
    pub fn scale_factor(mut self, val: f64) -> Self {
        self.scale_factor = Some(val);
        self
    }

    /// Sets per-source point bearings to constrain the direction snapping to
    /// the node.
    ///
    /// Each bearing must correspond to the source point at the same index. Passing
    /// None allows snapping in any direction.
    pub fn source_bearings(mut self, source_bearings: &'a [Option<Bearing>]) -> Self {
        self.source_bearings = Some(source_bearings);
        self
    }

    /// Sets per-destination point bearings to constrain the direction snapping to
    /// the node.
    ///
    /// Each bearing must correspond to the destination point at the same index. Passing
    /// None allows snapping in any direction.
    pub fn destination_bearings(mut self, destination_bearings: &'a [Option<Bearing>]) -> Self {
        self.destination_bearings = Some(destination_bearings);
        self
    }

    /// Sets the search radiuses for each source coordinate.
    ///
    /// Each radius must correspond to the source point at the same index. Radii must
    /// be positive. Passing None corresponds to an infinite search radius.
    pub fn source_radiuses(mut self, source_coordinate_radiuses: &'a [Option<f64>]) -> Self {
        self.source_radiuses = Some(source_coordinate_radiuses);
        self
    }

    /// Sets the search radiuses for each destination coordinate.
    ///
    /// Each radius must correspond to the source point at the same index. Radii must
    /// be positive. Passing None corresponds to an infinite search radius.
    pub fn destination_radiuses(
        mut self,
        destination_coordinate_radiuses: &'a [Option<f64>],
    ) -> Self {
        self.destination_radiuses = Some(destination_coordinate_radiuses);
        self
    }

    /// Enables or disables hint generation.
    pub fn generate_hints(mut self, generate_hints: bool) -> Self {
        self.generate_hints = generate_hints;
        self
    }

    /// Sets precomputed location hints for faster coordinate matching.
    ///
    /// Each hint corresponds to the destination point at the same index. OSRM will
    /// use the hint rather than the point information where supplied.
    ///
    /// Passing hints will result in radiuses, bearings,
    /// approaches being ignored for that point.
    pub fn source_hints(mut self, source_coordinate_hints: &'a [Option<&'a str>]) -> Self {
        self.source_hints = Some(source_coordinate_hints);
        self
    }

    /// Sets precomputed location hints for faster coordinate matching.
    ///
    /// Each hint corresponds to the source point at the same index. OSRM will
    /// use the hint rather than the point information where supplied.
    ///
    /// Passing hints will result in radiuses, bearings,
    /// approaches being ignored for that point.
    pub fn destination_hints(
        mut self,
        destination_coordinate_hints: &'a [Option<&'a str>],
    ) -> Self {
        self.destination_hints = Some(destination_coordinate_hints);
        self
    }

    /// Sets the approach direction for each source coordinate.
    pub fn source_approaches(mut self, source_approach_direction: &'a [Approach]) -> Self {
        self.source_approaches = Some(source_approach_direction);
        self
    }

    /// Sets the approach direction for each destination coordinate.
    pub fn destination_approaches(
        mut self,
        destination_approach_direction: &'a [Approach],
    ) -> Self {
        self.destination_approaches = Some(destination_approach_direction);
        self
    }

    /// Sets the exclusions (e.g., road classes) to apply to the request.
    pub fn exclude(mut self, exclude: &'a [Exclude]) -> Self {
        self.exclude = Some(exclude);
        self
    }

    /// Sets the snapping mode for matching coordinates to the road network.
    pub fn snapping(mut self, snapping: Snapping) -> Self {
        self.snapping = Some(snapping);
        self
    }

    /// Builds the [`TableRequest`], validating that all configuration is consistent.
    ///
    /// Returns an error if dimensions mismatch or invalid parameters are detected.
    /// See [`TableRequestError`] for all possible errors.
    pub fn build(&self) -> Result<TableRequest<'a>, TableRequestError> {
        if self.sources.is_empty() {
            return Err(TableRequestError::EmptySources);
        }
        if self.destinations.is_empty() {
            return Err(TableRequestError::EmptyDestinations);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(s) = self.fallback_speed {
            if s <= 0.0 {
                return Err(TableRequestError::NonPositiveFallbackSpeed);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(f) = self.scale_factor {
            if f <= 0.0 {
                return Err(TableRequestError::NonPositiveScaleFactor);
            }
        }

        if self.fallback_coordinate.is_some() ^ self.fallback_speed.is_some() {
            return Err(TableRequestError::IncompleteFallbackPair);
        }

        if self.scale_factor.is_some()
            && matches!(
                self.annotations,
                TableAnnotation::None | TableAnnotation::Distance
            )
        {
            return Err(TableRequestError::ScaleFactorRequiresDuration);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.source_bearings {
            if bearings.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(bearings) = self.destination_bearings {
            if bearings.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Bearings,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.source_radiuses {
            if radiuses.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
            if !radiuses.iter().all(|r| r.is_none_or(|v| v >= 0.0)) {
                return Err(TableRequestError::NegativeRadius);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(radiuses) = self.destination_radiuses {
            if radiuses.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Radiuses,
                ));
            }
            if !radiuses.iter().all(|r| r.is_none_or(|v| v >= 0.0)) {
                return Err(TableRequestError::NegativeRadius);
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.source_hints {
            if hints.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(hints) = self.destination_hints {
            if hints.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Hints,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.source_approaches {
            if approaches.len() != self.sources.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Approaches,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(approaches) = self.destination_approaches {
            if approaches.len() != self.destinations.len() {
                return Err(TableRequestError::DimensionMismatch(
                    DimensionMismatch::Approaches,
                ));
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(exclude) = self.exclude {
            if !exclude.is_empty() {
                if !match exclude[0] {
                    Exclude::Car(_) => exclude.iter().all(|e| matches!(e, Exclude::Car(_))),
                    Exclude::Bicycle(_) => exclude.iter().all(|e| matches!(e, Exclude::Bicycle(_))),
                } {
                    return Err(TableRequestError::DifferentExcludeTypes);
                }
            }
        }

        Ok(TableRequest {
            sources: self.sources,
            destinations: self.destinations,
            annotations: self.annotations,
            fallback_speed: self.fallback_speed,
            fallback_coordinate: self.fallback_coordinate,
            scale_factor: self.scale_factor,
            source_bearings: self.source_bearings,
            destination_bearings: self.destination_bearings,
            source_radiuses: self.source_radiuses,
            destination_radiuses: self.destination_radiuses,
            generate_hints: self.generate_hints,
            source_hints: self.source_hints,
            destination_hints: self.destination_hints,
            source_approaches: self.source_approaches,
            destination_approaches: self.destination_approaches,
            exclude: self.exclude,
            snapping: self.snapping,
        })
    }
}

/// The comprehensive error type returned when attempting to
/// construct an invalid [`TableRequest`].
#[derive(Error, Debug)]
pub enum TableRequestError {
    /// No sources in table request
    #[error("No sources in table request")]
    EmptySources,
    /// No destinations in table request
    #[error("No destinations in table request")]
    EmptyDestinations,
    /// Fallback speed must be greater than 0.0
    #[error("Fallback speed must be greater than 0.0")]
    NonPositiveFallbackSpeed,
    /// Scale factor must be greater than 0.0
    #[error("Scale factor must be greater than 0.0")]
    NonPositiveScaleFactor,
    /// Fallback speed and coordinate are co-dependent and must both be
    /// specified, or both null
    #[error(
        "Fallback speed and coordinate are co-dependent and must both be specified, or both null"
    )]
    IncompleteFallbackPair,
    /// Scale factor multiplies duration, so annotations must be
    /// TableAnnotation::Duration, or TableAnnotation::All
    #[error(
        "Scale factor multiplies duration, so annotations must be TableAnnotation::Duration, or TableAnnotation::All"
    )]
    ScaleFactorRequiresDuration,
    /// Mismatch of number of elements between points and one
    /// of the array-like options.
    #[error("Mismatch of dimensions between Points and {0:?}")]
    DimensionMismatch(DimensionMismatch),
    /// Cannot mix excludes of different [`Exclude`] variants.
    #[error("Exclude types are not all of the same type")]
    DifferentExcludeTypes,
    /// Radius values must be non-negative.
    #[error("Radii must be non-negative")]
    NegativeRadius,
}

/// Which metrics should the table service calculate.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum TableAnnotation {
    None = 0,
    Duration = 1,
    Distance = 2,
    /// Distance and Duration
    All = 3,
}
impl TableAnnotation {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Duration => "duration",
            Self::Distance => "distance",
            Self::All => "duration,distance",
        }
    }
}

/// If routing between two coordinates is impossible, a
/// a distance may be calculated as the crow flies, rather
/// than along the road network. This specifies whether the
/// raw, unsnapped input coordinate should be used or whether
/// the (lat,long) of the coordinate after snapping to calculate
/// that distance.
///
/// This argument is passed along with `fallback_speed` into
/// [`TableRequestBuilder::fallback`] where the `fallback_speed`
/// is used to calculate the duration.
///
/// Implements [`Debug`] if the `feature="debug"` feature flag
/// is set.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum TableFallbackCoordinate {
    /// Use the unsnapped input coordinate to calculate distance as
    /// the crow flies when routing fails.
    Input = 0,
    /// Use the snapped coordinate to calculate distance as the crow
    /// flies when routing fails.
    Snapped = 1,
}
impl TableFallbackCoordinate {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Snapped => "snapped",
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
#[allow(dead_code)]
pub struct TableResponse {
    /// If the request was successful "Ok" otherwise see the service dependent and general status codes
    pub code: String,
    pub destinations: Vec<TableLocationEntry>,
    /// Array of arrays that stores the matrix in row-major order. `durations[i][j]` gives the travel time from the i-th source to the j-th destination. Values are given in seconds. Can be `null` if no route between `i` and `j` can be found
    pub durations: Option<Vec<Vec<Option<f64>>>>,
    /// Array of arrays that stores the matrix in row-major order. `distances[i][j]` gives the travel distance from the i-th source to the j-th destination. Values are given in meters. Can be `null` if no route between `i` and `j` can be found
    pub distances: Option<Vec<Vec<Option<f64>>>>,
    pub sources: Vec<TableLocationEntry>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(
    any(feature = "native", feature = "remote"),
    derive(serde::Deserialize)
)]
pub struct TableLocationEntry {
    /// Hint is only null when generate_hints=false
    pub hint: Option<String>,
    pub location: [f64; 2],
    pub name: String,
    pub distance: f64,
}
