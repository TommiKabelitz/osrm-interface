#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum GeometryType {
    Polyline = 0,
    Polyline6 = 1,
    GeoJSON = 2,
}
impl GeometryType {
    pub fn url_form(self) -> &'static str {
        match self {
            Self::GeoJSON => "geojson",
            Self::Polyline => "polyline",
            Self::Polyline6 => "polyline6",
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub enum OverviewZoom {
    Simplified = 0,
    Full = 1,
    False = 2,
}
impl OverviewZoom {
    pub fn url_form(&self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Simplified => "simplified",
            Self::False => "false",
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum Profile {
    Car,
    Bike,
    Foot,
}
impl Profile {
    pub fn url_form(self) -> &'static str {
        match self {
            Self::Bike => "bike",
            Self::Car => "car",
            Self::Foot => "foot",
        }
    }
}
