#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum GeometryType {
    Polyline = 0,
    Polyline6 = 1,
    GeoJSON = 2,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum OverviewZoom {
    Simplified = 0,
    Full = 1,
    False = 2,
}
