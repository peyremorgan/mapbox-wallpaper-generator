use anyhow::{anyhow, Result};

pub const MAPBOX_WORLD_TILE_SIZE: f64 = 512.0;
pub const LOGICAL_TILE_SIZE_PX: f64 = 1280.0;
pub const PHYSICAL_TILE_SIZE_PX: u32 = 2560;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileCenter {
    pub row: u32,
    pub col: u32,
    pub lon: f64,
    pub lat: f64,
}

pub fn lon_to_x_px(lon: f64, zoom: f64) -> f64 {
    (lon + 180.0) / 360.0 * MAPBOX_WORLD_TILE_SIZE * 2f64.powf(zoom)
}

pub fn lat_to_y_px(lat: f64, zoom: f64) -> f64 {
    let lat_rad = lat.to_radians();
    let merc = (std::f64::consts::PI / 4.0 + lat_rad / 2.0).tan().ln();
    (1.0 - merc / std::f64::consts::PI) / 2.0 * MAPBOX_WORLD_TILE_SIZE * 2f64.powf(zoom)
}

pub fn x_px_to_lon(x: f64, zoom: f64) -> f64 {
    x / (MAPBOX_WORLD_TILE_SIZE * 2f64.powf(zoom)) * 360.0 - 180.0
}

pub fn y_px_to_lat(y: f64, zoom: f64) -> f64 {
    let n = std::f64::consts::PI
        - (2.0 * std::f64::consts::PI * y) / (MAPBOX_WORLD_TILE_SIZE * 2f64.powf(zoom));
    n.sinh().atan().to_degrees()
}

pub fn tile_grid_centers(
    center_lon: f64,
    center_lat: f64,
    zoom: f64,
    cols: u32,
    rows: u32,
) -> Result<Vec<TileCenter>> {
    if !(0.0..=22.0).contains(&zoom) {
        return Err(anyhow!("zoom must be between 0 and 22"));
    }
    if cols == 0 || rows == 0 {
        return Err(anyhow!("rows and cols must be positive"));
    }
    if !(-180.0..=180.0).contains(&center_lon) {
        return Err(anyhow!("longitude must be within [-180, 180]"));
    }
    if !(-85.0511..=85.0511).contains(&center_lat) {
        return Err(anyhow!("latitude must be within [-85.0511, 85.0511]"));
    }

    let center_x = lon_to_x_px(center_lon, zoom);
    let center_y = lat_to_y_px(center_lat, zoom);

    let col_mid = (cols as f64 - 1.0) / 2.0;
    let row_mid = (rows as f64 - 1.0) / 2.0;

    let mut tiles = Vec::with_capacity((cols * rows) as usize);
    for row in 0..rows {
        for col in 0..cols {
            let dx = (col as f64 - col_mid) * LOGICAL_TILE_SIZE_PX;
            let dy = (row as f64 - row_mid) * LOGICAL_TILE_SIZE_PX;
            let tile_x = center_x + dx;
            let tile_y = center_y + dy;
            tiles.push(TileCenter {
                row,
                col,
                lon: x_px_to_lon(tile_x, zoom),
                lat: y_px_to_lat(tile_y, zoom),
            });
        }
    }

    Ok(tiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() <= epsilon, "{a} != {b} (eps {epsilon})");
    }

    #[test]
    fn lon_roundtrip_is_stable() {
        let lon = 3.2728;
        let z = 8.69;
        approx_eq(x_px_to_lon(lon_to_x_px(lon, z), z), lon, 1e-9);
    }

    #[test]
    fn lat_roundtrip_is_stable() {
        let lat = 44.8263;
        let z = 10.0;
        approx_eq(y_px_to_lat(lat_to_y_px(lat, z), z), lat, 1e-9);
    }

    #[test]
    fn adjacent_tile_centers_are_1280_logical_px_apart() {
        let zoom = 12.0;
        let center_lon = 2.3522;
        let center_lat = 48.8566;
        let grid = tile_grid_centers(center_lon, center_lat, zoom, 3, 1).unwrap();
        let left = grid.iter().find(|t| t.col == 0).unwrap();
        let mid = grid.iter().find(|t| t.col == 1).unwrap();
        let right = grid.iter().find(|t| t.col == 2).unwrap();

        let left_x = lon_to_x_px(left.lon, zoom);
        let mid_x = lon_to_x_px(mid.lon, zoom);
        let right_x = lon_to_x_px(right.lon, zoom);

        approx_eq(mid_x - left_x, LOGICAL_TILE_SIZE_PX, 1e-6);
        approx_eq(right_x - mid_x, LOGICAL_TILE_SIZE_PX, 1e-6);
    }

    #[test]
    fn returns_expected_tile_count() {
        let grid = tile_grid_centers(0.0, 0.0, 5.0, 5, 3).unwrap();
        assert_eq!(grid.len(), 15);
    }

    #[test]
    fn rejects_invalid_zoom() {
        let result = tile_grid_centers(0.0, 0.0, 25.0, 5, 3);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_lat_lon_and_dimensions() {
        assert!(tile_grid_centers(181.0, 0.0, 5.0, 1, 1).is_err());
        assert!(tile_grid_centers(0.0, 90.0, 5.0, 1, 1).is_err());
        assert!(tile_grid_centers(0.0, 0.0, 5.0, 0, 1).is_err());
        assert!(tile_grid_centers(0.0, 0.0, 5.0, 1, 0).is_err());
    }
}
