use anyhow::{Context, Result};
use clap::Parser;
use mapbox_wallpaper_generator::cli::CliArgs;
use mapbox_wallpaper_generator::downloader::{
    build_mapbox_client, download_tiles, DownloadOptions, DEFAULT_PLAYGROUND_TOKEN,
};
use mapbox_wallpaper_generator::geocoder::{build_nominatim_client, geocode_place};
use mapbox_wallpaper_generator::stitcher::{save_rgb_image, stitch_tiles};
use mapbox_wallpaper_generator::tile_math::{tile_grid_centers, PHYSICAL_TILE_SIZE_PX};
use std::time::Instant;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = CliArgs::parse();
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| args.default_output_path());

    let token = args
        .token
        .clone()
        .unwrap_or_else(|| DEFAULT_PLAYGROUND_TOKEN.to_string());

    let started = Instant::now();

    let geocoder_client = build_nominatim_client()?;
    let geocode = geocode_place(&geocoder_client, &args.place)
        .with_context(|| format!("failed to geocode '{}': no location found", args.place))?;
    println!(
        "Resolved '{}' to: {} ({:.6}, {:.6})",
        args.place, geocode.display_name, geocode.lat, geocode.lon
    );

    let grid = tile_grid_centers(geocode.lon, geocode.lat, args.zoom, args.cols, args.rows)?;
    println!(
        "Downloading {} tiles at zoom {:.2} ({}x{} layout)...",
        grid.len(),
        args.zoom,
        args.cols,
        args.rows
    );

    let mapbox_client = build_mapbox_client()?;
    let options = DownloadOptions {
        zoom: args.zoom,
        token,
        concurrency: args.concurrency,
    };
    let tiles = download_tiles(&mapbox_client, &grid, &options)?;

    println!("Stitching tiles...");
    let stitched = stitch_tiles(tiles, args.cols, args.rows)?;
    save_rgb_image(&stitched, &output_path, 95)?;

    println!(
        "Saved {} ({}x{}) in {:.2}s",
        output_path.display(),
        args.cols * PHYSICAL_TILE_SIZE_PX,
        args.rows * PHYSICAL_TILE_SIZE_PX,
        started.elapsed().as_secs_f64(),
    );
    Ok(())
}
