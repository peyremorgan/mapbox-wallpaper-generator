use anyhow::{anyhow, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::imageops;
use image::{DynamicImage, ImageFormat, RgbImage};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::downloader::DownloadedTile;
use crate::tile_math::PHYSICAL_TILE_SIZE_PX;

pub fn stitch_tiles(tiles: Vec<DownloadedTile>, cols: u32, rows: u32) -> Result<RgbImage> {
    stitch_tiles_with_size(tiles, cols, rows, PHYSICAL_TILE_SIZE_PX)
}

fn stitch_tiles_with_size(
    tiles: Vec<DownloadedTile>,
    cols: u32,
    rows: u32,
    tile_size_px: u32,
) -> Result<RgbImage> {
    if cols == 0 || rows == 0 {
        return Err(anyhow!("rows and cols must be positive"));
    }

    let expected_tiles = (cols * rows) as usize;
    if tiles.len() != expected_tiles {
        return Err(anyhow!(
            "tile count mismatch: expected {}, got {}",
            expected_tiles,
            tiles.len()
        ));
    }

    let mut by_position = HashMap::new();
    for tile in tiles {
        by_position.insert((tile.row, tile.col), tile.bytes);
    }

    let width = cols * tile_size_px;
    let height = rows * tile_size_px;
    let mut canvas = RgbImage::new(width, height);

    for row in 0..rows {
        for col in 0..cols {
            let bytes = by_position
                .get(&(row, col))
                .ok_or_else(|| anyhow!("missing tile at ({row}, {col})"))?;

            let image = decode_tile(bytes).context("failed to decode tile image")?;
            if image.width() != tile_size_px || image.height() != tile_size_px {
                return Err(anyhow!(
                    "tile dimensions mismatch at ({}, {}): expected {}x{}, got {}x{}",
                    row,
                    col,
                    tile_size_px,
                    tile_size_px,
                    image.width(),
                    image.height()
                ));
            }

            imageops::replace(
                &mut canvas,
                &image,
                (col * tile_size_px) as i64,
                (row * tile_size_px) as i64,
            );
        }
    }

    Ok(canvas)
}

fn decode_tile(bytes: &[u8]) -> Result<RgbImage> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Jpeg)
        .or_else(|_| image::load_from_memory(bytes))
        .context("failed to decode image bytes")?;
    Ok(match image {
        DynamicImage::ImageRgb8(img) => img,
        _ => image.to_rgb8(),
    })
}

pub fn save_rgb_image(image: &RgbImage, output_path: &Path, quality: u8) -> Result<()> {
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "jpg".to_string());

    match ext.as_str() {
        "jpg" | "jpeg" => {
            let file = File::create(output_path).with_context(|| {
                format!("failed to create output file at {}", output_path.display())
            })?;
            let mut writer = std::io::BufWriter::new(file);
            let mut encoder = JpegEncoder::new_with_quality(&mut writer, quality);
            encoder
                .encode_image(image)
                .with_context(|| format!("failed to encode JPEG at {}", output_path.display()))
        }
        "png" => image
            .save_with_format(output_path, ImageFormat::Png)
            .with_context(|| format!("failed to encode PNG at {}", output_path.display())),
        _ => Err(anyhow!(
            "unsupported output extension '{}', use .jpg/.jpeg or .png",
            ext
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloader::DownloadedTile;
    use image::{ImageBuffer, Rgb};
    use std::fs;

    fn jpeg_bytes(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
        let img: RgbImage = ImageBuffer::from_pixel(width, height, Rgb(color));
        let dyn_img = DynamicImage::ImageRgb8(img);
        let mut bytes = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut bytes, 95);
        encoder.encode_image(&dyn_img).unwrap();
        bytes
    }

    #[test]
    fn stitches_small_grid_correctly() {
        let s = 4;
        let tiles = vec![
            DownloadedTile {
                row: 0,
                col: 0,
                bytes: jpeg_bytes(s, s, [255, 0, 0]),
            },
            DownloadedTile {
                row: 0,
                col: 1,
                bytes: jpeg_bytes(s, s, [0, 255, 0]),
            },
            DownloadedTile {
                row: 1,
                col: 0,
                bytes: jpeg_bytes(s, s, [0, 0, 255]),
            },
            DownloadedTile {
                row: 1,
                col: 1,
                bytes: jpeg_bytes(s, s, [255, 255, 0]),
            },
        ];

        let stitched = stitch_tiles_with_size(tiles, 2, 2, s).unwrap();
        assert_eq!(stitched.width(), 8);
        assert_eq!(stitched.height(), 8);

        let tl = stitched.get_pixel(1, 1).0;
        let tr = stitched.get_pixel(6, 1).0;
        let bl = stitched.get_pixel(1, 6).0;
        let br = stitched.get_pixel(6, 6).0;

        assert!(tl[0] > tl[1] && tl[0] > tl[2]);
        assert!(tr[1] > tr[0] && tr[1] > tr[2]);
        assert!(bl[2] > bl[0] && bl[2] > bl[1]);
        assert!(br[0] > 200 && br[1] > 200);
    }

    #[test]
    fn rejects_wrong_tile_count() {
        let tiles = vec![DownloadedTile {
            row: 0,
            col: 0,
            bytes: jpeg_bytes(2, 2, [1, 2, 3]),
        }];
        let stitched = stitch_tiles_with_size(tiles, 2, 2, 2);
        assert!(stitched.is_err());
    }

    #[test]
    fn rejects_wrong_tile_dimensions() {
        let tiles = vec![DownloadedTile {
            row: 0,
            col: 0,
            bytes: jpeg_bytes(3, 3, [10, 10, 10]),
        }];
        let stitched = stitch_tiles_with_size(tiles, 1, 1, 2);
        assert!(stitched.is_err());
    }

    #[test]
    fn saves_jpeg_and_png_by_extension() {
        let img: RgbImage = ImageBuffer::from_pixel(2, 2, Rgb([42, 42, 42]));
        let base = std::env::temp_dir().join(format!(
            "mapbox_wallpaper_generator_test_{}",
            std::process::id()
        ));
        let jpg_path = base.with_extension("jpg");
        let png_path = base.with_extension("png");

        save_rgb_image(&img, &jpg_path, 90).unwrap();
        save_rgb_image(&img, &png_path, 90).unwrap();

        let jpg = fs::read(&jpg_path).unwrap();
        let png = fs::read(&png_path).unwrap();
        assert_eq!(&jpg[0..2], &[0xFF, 0xD8]);
        assert_eq!(
            &png[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );

        let _ = fs::remove_file(jpg_path);
        let _ = fs::remove_file(png_path);
    }
}
