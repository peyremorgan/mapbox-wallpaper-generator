use anyhow::{anyhow, Context, Result};
use reqwest::blocking::Client;
use reqwest::Url;
use serde::Deserialize;

const NOMINATIM_ENDPOINT: &str = "https://nominatim.openstreetmap.org/search";

#[derive(Debug, Clone, PartialEq)]
pub struct GeocodeResult {
    pub lat: f64,
    pub lon: f64,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NominatimResultItem {
    lat: String,
    lon: String,
    display_name: String,
}

pub fn build_nominatim_client() -> Result<Client> {
    Client::builder()
        .user_agent("mapbox-wallpaper-generator/0.1 (+https://openstreetmap.org)")
        .build()
        .context("failed to build Nominatim HTTP client")
}

pub fn parse_nominatim_response(response_body: &str) -> Result<GeocodeResult> {
    let mut items: Vec<NominatimResultItem> =
        serde_json::from_str(response_body).context("failed to parse Nominatim JSON response")?;

    let first = items
        .drain(..)
        .next()
        .ok_or_else(|| anyhow!("no geocoding results returned"))?;

    let lat = first
        .lat
        .parse::<f64>()
        .context("failed to parse latitude from Nominatim")?;
    let lon = first
        .lon
        .parse::<f64>()
        .context("failed to parse longitude from Nominatim")?;

    Ok(GeocodeResult {
        lat,
        lon,
        display_name: first.display_name,
    })
}

pub fn geocode_place(client: &Client, place: &str) -> Result<GeocodeResult> {
    let mut url = Url::parse(NOMINATIM_ENDPOINT)?;
    url.query_pairs_mut()
        .append_pair("q", place)
        .append_pair("format", "jsonv2")
        .append_pair("limit", "1")
        .append_pair("featureType", "city");

    let response = client
        .get(url)
        .send()
        .context("failed to call Nominatim API")?
        .error_for_status()
        .context("Nominatim returned non-success status")?;

    let body = response
        .text()
        .context("failed to read Nominatim response body")?;
    parse_nominatim_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_nominatim_payload() {
        let json = r#"[
          {
            "lat": "44.8263",
            "lon": "3.2728",
            "display_name": "Some Place"
          }
        ]"#;

        let parsed = parse_nominatim_response(json).unwrap();
        assert_eq!(parsed.display_name, "Some Place");
        assert_eq!(parsed.lat, 44.8263);
        assert_eq!(parsed.lon, 3.2728);
    }

    #[test]
    fn fails_on_empty_payload() {
        let parsed = parse_nominatim_response("[]");
        assert!(parsed.is_err());
    }

    #[test]
    fn fails_on_invalid_numeric_values() {
        let json = r#"[{"lat":"oops","lon":"3.2728","display_name":"Bad"}]"#;
        let parsed = parse_nominatim_response(json);
        assert!(parsed.is_err());
    }
}
