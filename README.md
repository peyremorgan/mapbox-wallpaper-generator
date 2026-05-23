# MapBox Wallpaper Generator

A command-line tool that generates ultra high-resolution satellite wallpapers by
stitching tiles from the [Mapbox Static Images API](https://docs.mapbox.com/api/maps/static-images/).
Give it a place name; it geocodes the location via
[OpenStreetMap Nominatim](https://nominatim.org/), downloads a configurable grid of
`1280×1280 @2x` satellite tiles (each rendered at **2560×2560** pixels), stitches them
seamlessly into a single image, and saves the result.

Default settings produce a **12 800×7 680** landscape image (5 columns × 3 rows) in
JPEG format.  Output format is inferred from the file extension — `.jpg`/`.jpeg` or
`.png` are both supported.

## Usage

```
mapbox-wallpaper <PLACE> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<PLACE>` | Place name to geocode, e.g. `"Paris"` or `"Tokyo, Japan"` |

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--zoom <ZOOM>` | `12.0` | Mapbox zoom level (0 – 22). Higher values show more detail over a smaller area. |
| `--cols <COLS>` | `5` | Number of tile columns in the output mosaic. |
| `--rows <ROWS>` | `3` | Number of tile rows in the output mosaic. |
| `--output <PATH>` | `<place>_z<zoom>.jpg` | Output image path. Extension determines format (`.jpg` or `.png`). |
| `--token <TOKEN>` | env `MAPBOX_TOKEN` / playground token | Mapbox API access token. |
| `--concurrency <N>` | `4` | Parallel tile downloads (1 – 32). |

### Examples

```sh
# 12 800×7 680 JPEG of Paris at the default zoom
mapbox-wallpaper "Paris"

# Higher zoom — more street-level detail, smaller geographic area
mapbox-wallpaper "Tokyo, Japan" --zoom 14

# 5×5 square mosaic saved as lossless PNG
mapbox-wallpaper "New York" --cols 5 --rows 5 --output newyork.png

# Custom Mapbox token via flag
mapbox-wallpaper "Berlin" --token sk.ey...

# Custom Mapbox token via environment variable
MAPBOX_TOKEN=sk.ey... mapbox-wallpaper "Sydney"
```

### Output resolution

Each tile is fetched at `1280×1280@2x`, yielding **2560×2560** pixels per tile.
Final canvas dimensions are `cols × 2560` by `rows × 2560`.

| Grid | Width | Height |
|------|------:|-------:|
| 5×3 (default) | 12 800 px | 7 680 px |
| 5×5 | 12 800 px | 12 800 px |
| 7×4 | 17 920 px | 10 240 px |

### Mapbox token

A Mapbox playground token is bundled for quick prototyping; it requires the
`Referer: https://docs.mapbox.com/` header that the tool sends automatically.
For production use, create a free account at
[mapbox.com](https://account.mapbox.com/auth/signup/) — the free tier includes
50 000 Static Images requests per month.

## Contributing

### Requirements

- **Rust stable ≥ 1.95** — the project ships a `rust-toolchain.toml` that pins the
  toolchain, so [rustup](https://rustup.rs/) is the recommended way to install Rust.
  `rustfmt` and `clippy` are declared as required components and will be installed
  automatically by rustup.
- No other system dependencies are required; all crates are pure Rust or link to
  system TLS (via `rustls-tls`, no OpenSSL needed).

### Building

```sh
# Debug build
cargo build

# Optimised release build (recommended for actual wallpaper generation)
cargo build --release

# The release binary lands at:
./target/release/mapbox_wallpaper_generator
```

### Testing

```sh
# Run the full test suite (unit + integration)
cargo test

# Run only the CLI integration tests
cargo test --test e2e_cli

# Run only a specific module's tests
cargo test tile_math::tests

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format check (no writes)
cargo fmt --all -- --check

# Format and apply
cargo fmt --all
```

The test suite is fully offline — no network calls are made. Live end-to-end
behaviour (real Nominatim + Mapbox requests) can be exercised by running the binary
directly as shown in the [Usage](#usage) examples above.

## Disclaimer

This project is an independent work and is not affiliated with, endorsed by, or sponsored by Mapbox.
Mapbox is a registered trademark of Mapbox, Inc.