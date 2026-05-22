use anyhow::{anyhow, Context, Result};
use rayon::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, CONNECTION, PRAGMA, REFERER,
    USER_AGENT,
};
use reqwest::StatusCode;
use std::thread;
use std::time::Duration;

use crate::tile_math::TileCenter;

pub const DEFAULT_PLAYGROUND_TOKEN: &str =
    "REPLACE_WITH_MAPBOX_TOKEN";

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub zoom: f64,
    pub token: String,
    pub concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct DownloadedTile {
    pub row: u32,
    pub col: u32,
    pub bytes: Vec<u8>,
}

pub fn build_mapbox_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0",
        ),
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "image/avif,image/webp,image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5",
        ),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://docs.mapbox.com/"),
    );
    headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));

    Client::builder()
        .default_headers(headers)
        .build()
        .context("failed to build Mapbox HTTP client")
}

pub fn mapbox_static_url(lon: f64, lat: f64, zoom: f64, token: &str) -> String {
    format!(
        "https://api.mapbox.com/styles/v1/mapbox/satellite-v9/static/{lon:.6},{lat:.6},{zoom:.2},0/1280x1280@2x?attribution=false&logo=false&access_token={token}"
    )
}

fn fetch_tile(
    client: &Client,
    tile: &TileCenter,
    options: &DownloadOptions,
) -> Result<DownloadedTile> {
    let url = mapbox_static_url(tile.lon, tile.lat, options.zoom, &options.token);
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 1..=2 {
        match client.get(&url).send() {
            Ok(response) => {
                let status = response.status();
                if status == StatusCode::OK {
                    let bytes = response
                        .bytes()
                        .context("failed to read tile response body")?
                        .to_vec();
                    return Ok(DownloadedTile {
                        row: tile.row,
                        col: tile.col,
                        bytes,
                    });
                }

                last_error = Some(anyhow!(
                    "Mapbox returned status {} for tile ({}, {})",
                    status,
                    tile.row,
                    tile.col
                ));
            }
            Err(err) => {
                last_error = Some(anyhow!(err).context(format!(
                    "request failed for tile ({}, {}), attempt {}",
                    tile.row, tile.col, attempt
                )));
            }
        }

        thread::sleep(Duration::from_millis(300));
    }

    Err(last_error.unwrap_or_else(|| anyhow!("unknown tile download error")))
}

pub fn download_tiles(
    client: &Client,
    tiles: &[TileCenter],
    options: &DownloadOptions,
) -> Result<Vec<DownloadedTile>> {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.concurrency)
        .build()
        .context("failed to build rayon thread pool")?;

    pool.install(|| {
        tiles
            .par_iter()
            .map(|tile| fetch_tile(client, tile, options))
            .collect::<Result<Vec<_>>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_url_contains_expected_parts() {
        let url = mapbox_static_url(3.2728, 44.8263, 8.69, "TOKEN123");
        assert!(url.contains("mapbox/satellite-v9"));
        assert!(url.contains("/1280x1280@2x"));
        assert!(url.contains("access_token=TOKEN123"));
        assert!(url.contains("3.272800,44.826300,8.69,0"));
    }
}
