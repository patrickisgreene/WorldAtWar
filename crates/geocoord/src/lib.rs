use bevy_derive::{Deref, DerefMut};
use bevy_math::{DVec2, DVec3, prelude::*};
use bevy_reflect::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Geographic Latitude & Longtitude coordinate.
#[derive(Reflect, Debug, PartialEq, Clone, Copy, Deref, DerefMut, Serialize, Deserialize)]
pub struct GeoCoord(DVec2);

impl fmt::Display for GeoCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}°, {}°)", self.0.x, self.0.y)
    }
}

/// Constants
impl GeoCoord {
    pub const PARIS: Self = Self(DVec2::new(48.864716, 2.349014));
    pub const LONDON: Self = Self(DVec2::new(51.50722, -0.1275));
    pub const MADRID: Self = Self(DVec2::new(40.41889, -3.69194));
    pub const BERLIN: Self = Self(DVec2::new(52.504043, 13.393236));
    pub const KYIV: Self = Self(DVec2::new(50.45466, 30.5238));
    pub const MOSCOW: Self = Self(DVec2::new(55.751244, 37.618423));
    pub const CAIRO: Self = Self(DVec2::new(30.033333, 31.233334));
    pub const PERTH: Self = Self(DVec2::new(-31.9522, 115.8614));
    pub const TOKYO: Self = Self(DVec2::new(35.652832, 139.839478));

    pub const HOUSTON: Self = Self(DVec2::new(29.749907, -95.358421));
    pub const DALLAS: Self = Self(DVec2::new(32.779167, -96.808891));
    pub const MIAMI: Self = Self(DVec2::new(25.761681, -80.191788));
    pub const WASHINGTON_DC: Self = Self(DVec2::new(38.895, -77.0366));
    pub const SEATTLE: Self = Self(DVec2::new(47.608013, -122.335167));
    pub const MEMPHIS: Self = Self(DVec2::new(35.117500, -89.971107));
}

/// Constructors
impl GeoCoord {
    /// Create a Coordinate from a raw `DVec2` Lat/Long vector
    pub fn new(raw: DVec2) -> GeoCoord {
        Self(raw)
    }

    /// Create a coordinate from a latitude and a longitude
    pub fn from_gps(lat: f64, long: f64) -> Self {
        Self(DVec2::new(lat, long))
    }

    /// Calculate the Coordinate from a position on earths surface.
    pub fn from_world(pos: DVec3) -> Self {
        let x = pos.x;
        let y = pos.y;
        let z = pos.z;

        let radius = pos.length();
        let lat_rad = (y / radius).asin();
        let lon_rad = -z.atan2(x);

        // Round to 10 decimal places to avoid floating-point precision errors
        // This is approximately 1.1 cm precision at the equator, which is more than sufficient
        let lat = (lat_rad.to_degrees() * 1e10).round() / 1e10;
        let lon = (lon_rad.to_degrees() * 1e10).round() / 1e10;

        Self::from_gps(lat, lon)
    }
}

// Conversions
impl GeoCoord {
    /// Convert a lat//long (in degrees) to a position on a sphere
    /// with the radius of earths surface (6_371_000.0).
    pub fn world_pos(&self) -> DVec3 {
        let earth_radius = 6_371_000.0;

        let lat_rad = self.0.x.to_radians();
        let lon_rad = self.0.y.to_radians();

        DVec3::new(
            earth_radius * lat_rad.cos() * (-lon_rad).cos(),
            earth_radius * lat_rad.sin(),
            earth_radius * lat_rad.cos() * (-lon_rad).sin(),
        )
    }

    /// Convert lat/long (in degrees) to UV coordinates for equirectangular projection
    /// Match the earth mesh UV conversion (mesh_gen.rs:201-202)
    /// Latitude: -90 to 90 → 1 to 0 (inverted because texture is top-down)
    /// Longitude: -180 to 180 → 0 to 1
    /// Negate longitude to match texture orientation
    pub fn uv(&self) -> Vec2 {
        let lat_rad = self.0.x.to_radians();
        let long_rad = (-self.0.y).to_radians();
        let u = 1.0 - ((long_rad / std::f64::consts::TAU) + 0.5) as f32;
        let v = 1.0 - ((lat_rad / std::f64::consts::PI) + 0.5) as f32;
        Vec2::new(u, v)
    }

    /// Calculate the surface rotation and return
    /// it as a Quaternion.
    pub fn rotation(&self) -> Quat {
        let surface_normal = self.world_pos().normalize();
        Quat::from_rotation_arc(Vec3::Y, surface_normal.as_vec3())
    }

    pub fn surface_normal(&self) -> DVec3 {
        self.world_pos().normalize()
    }
}

/// Math
impl GeoCoord {
    /// Calculate the great-circle distance between two coordinates using the Haversine formula.
    /// Returns the distance in meters.
    pub fn distance(&self, other: Self) -> f64 {
        let earth_radius = 6_371_000.0; // Earth's radius in meters

        let lat1 = self.0.x.to_radians();
        let lat2 = other.0.x.to_radians();
        let delta_lat = (other.0.x - self.0.x).to_radians();
        let delta_lon = (other.0.y - self.0.y).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        earth_radius * c
    }

    /// Calculate the midpoint between two coordinates on the Earth's surface.
    /// Uses the spherical midpoint formula to account for Earth's curvature.
    pub fn midpoint(&self, other: GeoCoord) -> GeoCoord {
        let lat1 = self.0.x.to_radians();
        let lon1 = self.0.y.to_radians();
        let lat2 = other.0.x.to_radians();
        let lon2 = other.0.y.to_radians();

        let delta_lon = lon2 - lon1;

        let bx = lat2.cos() * delta_lon.cos();
        let by = lat2.cos() * delta_lon.sin();

        let lat_mid =
            (lat1.sin() + lat2.sin()).atan2(((lat1.cos() + bx).powi(2) + by.powi(2)).sqrt());
        let lon_mid = lon1 + by.atan2(lat1.cos() + bx);

        Self::from_gps(lat_mid.to_degrees(), lon_mid.to_degrees())
    }

    /// Spherical linear interpolation (slerp) between two coordinates along the great circle.
    /// Parameter t should be in the range [0.0, 1.0], where 0.0 returns self and 1.0 returns to.
    pub fn lerp_to(&self, to: GeoCoord, t: f64) -> GeoCoord {
        let lat1 = self.0.x.to_radians();
        let lon1 = self.0.y.to_radians();
        let lat2 = to.0.x.to_radians();
        let lon2 = to.0.y.to_radians();

        let delta_lon = lon2 - lon1;

        // Calculate the angular distance between the points
        let cos_dist = lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * delta_lon.cos();
        let dist = cos_dist.acos();

        // Handle the case where points are very close (avoid division by zero)
        if dist.abs() < 1e-10 {
            return *self;
        }

        let sin_dist = dist.sin();
        let a = ((1.0 - t) * dist).sin() / sin_dist;
        let b = (t * dist).sin() / sin_dist;

        let x = a * lat1.cos() * lon1.cos() + b * lat2.cos() * lon2.cos();
        let y = a * lat1.cos() * lon1.sin() + b * lat2.cos() * lon2.sin();
        let z = a * lat1.sin() + b * lat2.sin();

        let lat_result = z.atan2((x * x + y * y).sqrt());
        let lon_result = y.atan2(x);

        Self::from_gps(lat_result.to_degrees(), lon_result.to_degrees())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_position_round_trip() {
        let cities = vec![
            GeoCoord::MOSCOW,
            GeoCoord::MIAMI,
            GeoCoord::SEATTLE,
            GeoCoord::PARIS,
            GeoCoord::PERTH,
            GeoCoord::LONDON,
        ];
        for city in cities {
            assert_eq!(city, GeoCoord::from_world(city.world_pos()),);
        }
    }

    #[test]
    fn distance_check() {
        let distance = GeoCoord::MIAMI.distance(GeoCoord::PARIS);
        assert_eq!(distance, 7356146.5510973865);
    }

    #[test]
    fn midpoint_check() {
        // Test midpoint between Paris and London
        let paris = GeoCoord::PARIS;
        let london = GeoCoord::LONDON;
        let midpoint = paris.midpoint(london);

        // The midpoint should be equidistant from both points
        let dist_to_paris = midpoint.distance(paris);
        let dist_to_london = midpoint.distance(london);
        let diff = (dist_to_paris - dist_to_london).abs();
        assert!(
            diff < 100.0,
            "Midpoint should be equidistant from both points. Difference: {} meters",
            diff
        );

        // The sum of distances from midpoint to both points should equal the total distance
        let total_distance = paris.distance(london);
        let sum_of_halves = dist_to_paris + dist_to_london;
        let distance_diff = (total_distance - sum_of_halves).abs();
        assert!(
            distance_diff < 1.0,
            "Sum of half distances should match total. Diff: {} meters",
            distance_diff
        );

        // Test midpoint is symmetric (a.midpoint(b) == b.midpoint(a))
        let midpoint_reverse = london.midpoint(paris);
        let lat_diff = (midpoint.0.x - midpoint_reverse.0.x).abs();
        let lon_diff = (midpoint.0.y - midpoint_reverse.0.y).abs();
        assert!(lat_diff < 0.0001, "Midpoint should be symmetric (latitude)");
        assert!(
            lon_diff < 0.0001,
            "Midpoint should be symmetric (longitude)"
        );

        // Test with a longer distance (Miami to Tokyo)
        let miami = GeoCoord::MIAMI;
        let tokyo = GeoCoord::TOKYO;
        let mid_pacific = miami.midpoint(tokyo);

        let dist_miami = mid_pacific.distance(miami);
        let dist_tokyo = mid_pacific.distance(tokyo);
        let diff_pacific = (dist_miami - dist_tokyo).abs();
        assert!(
            diff_pacific < 100.0,
            "Midpoint across Pacific should be equidistant. Difference: {} meters",
            diff_pacific
        );

        // Test midpoint with the same point (should return the same point)
        let same_point = paris.midpoint(paris);
        assert_eq!(
            same_point, paris,
            "Midpoint of same point should return itself"
        );
    }

    #[test]
    fn test_lerp_to() {
        // Test that t=0.0 returns the starting point
        let start = GeoCoord::PARIS;
        let end = GeoCoord::LONDON;
        let result = start.lerp_to(end, 0.0);
        assert_eq!(result, start);

        // Test that t=1.0 returns approximately the ending point
        let result = start.lerp_to(end, 1.0);
        let diff_lat = (result.0.x - end.0.x).abs();
        let diff_lon = (result.0.y - end.0.y).abs();
        assert!(
            diff_lat < 0.0001,
            "Latitude difference too large: {}",
            diff_lat
        );
        assert!(
            diff_lon < 0.0001,
            "Longitude difference too large: {}",
            diff_lon
        );

        // Test that t=0.5 returns approximately the midpoint
        let midpoint = start.midpoint(end);
        let lerp_mid = start.lerp_to(end, 0.5);

        // Should be very close (within a small tolerance due to floating-point precision)
        let diff_lat = (midpoint.0.x - lerp_mid.0.x).abs();
        let diff_lon = (midpoint.0.y - lerp_mid.0.y).abs();
        assert!(
            diff_lat < 0.0001,
            "Latitude difference too large: {}",
            diff_lat
        );
        assert!(
            diff_lon < 0.0001,
            "Longitude difference too large: {}",
            diff_lon
        );

        // Test that interpolated points lie on the great circle path
        // The distances should add up correctly
        let quarter = start.lerp_to(end, 0.25);
        let total_distance = start.distance(end);
        let first_quarter = start.distance(quarter);
        let remaining = quarter.distance(end);

        // Check that the sum of distances is approximately equal to total distance
        let distance_sum = first_quarter + remaining;
        let diff = (total_distance - distance_sum).abs();
        assert!(
            diff < 1.0,
            "Distance sum should match total distance. Diff: {} meters",
            diff
        );

        // Verify the quarter point is approximately 25% of the way
        let ratio = first_quarter / total_distance;
        assert!(
            (ratio - 0.25).abs() < 0.001,
            "Quarter point ratio incorrect: {}",
            ratio
        );
    }
}
