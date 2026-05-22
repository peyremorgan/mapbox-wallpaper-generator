use clap::Parser;
use std::path::PathBuf;

fn parse_positive_u32(value: &str) -> Result<u32, String> {
    let parsed: u32 = value
        .parse()
        .map_err(|_| format!("invalid positive integer: {value}"))?;
    if parsed == 0 {
        return Err("value must be >= 1".to_string());
    }
    Ok(parsed)
}

fn parse_concurrency(value: &str) -> Result<usize, String> {
    let parsed: usize = value
        .parse()
        .map_err(|_| format!("invalid integer: {value}"))?;
    if !(1..=32).contains(&parsed) {
        return Err("concurrency must be between 1 and 32".to_string());
    }
    Ok(parsed)
}

#[derive(Debug, Clone, Parser)]
#[command(name = "mapbox-wallpaper")]
#[command(about = "Generate ultra high-resolution wallpapers by stitching Mapbox satellite tiles")]
pub struct CliArgs {
    /// Place name to geocode (for example: "Paris" or "Tokyo, Japan")
    pub place: String,

    /// Map zoom level used for all fetched tiles
    #[arg(long, default_value_t = 12.0)]
    pub zoom: f64,

    /// Number of tile columns in the output mosaic
    #[arg(long, default_value_t = 5, value_parser = parse_positive_u32)]
    pub cols: u32,

    /// Number of tile rows in the output mosaic
    #[arg(long, default_value_t = 3, value_parser = parse_positive_u32)]
    pub rows: u32,

    /// Output image path (defaults to <place>_z<zoom>.jpg)
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Mapbox token. If omitted, reads MAPBOX_TOKEN or uses the playground token.
    #[arg(long, env = "MAPBOX_TOKEN")]
    pub token: Option<String>,

    /// Maximum parallel tile downloads
    #[arg(long, default_value_t = 4, value_parser = parse_concurrency)]
    pub concurrency: usize,
}

impl CliArgs {
    pub fn default_output_path(&self) -> PathBuf {
        let mut sanitized = String::with_capacity(self.place.len());
        for c in self.place.chars() {
            if c.is_ascii_alphanumeric() {
                sanitized.push(c.to_ascii_lowercase());
            } else if c == ' ' || c == '-' || c == '_' {
                sanitized.push('_');
            }
        }

        if sanitized.is_empty() {
            sanitized.push_str("wallpaper");
        }

        while sanitized.contains("__") {
            sanitized = sanitized.replace("__", "_");
        }

        PathBuf::from(format!(
            "{}_z{:.2}.jpg",
            sanitized.trim_matches('_'),
            self.zoom
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::CliArgs;
    use clap::Parser;

    #[test]
    fn builds_default_output_path() {
        let args = CliArgs::parse_from(["bin", "New York"]);
        assert_eq!(
            args.default_output_path().to_string_lossy(),
            "new_york_z12.00.jpg"
        );
    }

    #[test]
    fn validates_positive_rows_and_cols() {
        let parsed = CliArgs::try_parse_from(["bin", "Paris", "--rows", "0"]);
        assert!(parsed.is_err());
    }

    #[test]
    fn validates_concurrency_bounds() {
        let low = CliArgs::try_parse_from(["bin", "Paris", "--concurrency", "0"]);
        let high = CliArgs::try_parse_from(["bin", "Paris", "--concurrency", "64"]);
        assert!(low.is_err());
        assert!(high.is_err());
    }
}
