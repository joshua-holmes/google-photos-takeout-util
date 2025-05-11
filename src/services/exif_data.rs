use std::{path::Path, str::FromStr};

use chrono;
use little_exif::{exif_tag::ExifTag, metadata::Metadata};
use serde::{Deserialize, Serialize};

static EXIF_TIMESTAMP_FMT: &str = "%Y:%m:%d %H:%M:%S%z";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeoutExif {
    title: Option<String>,
    description: Option<String>,
    creation_time: Option<TimeStamp>,
    photo_taken_time: Option<TimeStamp>,
    geo_data: Option<GeoData>,
    people: Option<Vec<Person>>,
    url: Option<String>,
}
impl TakeoutExif {
    pub fn apply_to_image(&self, path: &Path) {
        let mut tags = Vec::new();
        if let Some(description) = self.description.clone() {
            tags.push(ExifTag::ImageDescription(description));
        }
        if let Some(timestamp) = self.creation_time.as_ref().and_then(|t| t.to_datetime()) {
            let timestamp_formatted = timestamp.format(EXIF_TIMESTAMP_FMT).to_string();
            tags.push(ExifTag::DateTimeOriginal(timestamp_formatted.clone()));
            tags.push(ExifTag::CreateDate(timestamp_formatted.clone()));
            tags.push(ExifTag::ModifyDate(timestamp_formatted));
        }

        let mut metadata = Metadata::new();
        for t in tags {
            metadata.set_tag(t);
        }
        metadata.write_to_file(path);
    }

    pub fn from_json(value: &str) -> Result<Self, JsonParseError> {
        serde_json::from_str(value).map_err(JsonParseError::from)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeStamp {
    timestamp: Option<String>,
    formatted: Option<String>,
}
impl TimeStamp {
    /// Convert to [`chrono::DateTime`] if the necessary fields are present and parsing is successful.
    /// Otherwise, return `None`.
    // TODO: Maybe return `Result` instead for better communication about why parsing failed.
    fn to_datetime(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.timestamp
            .as_ref()
            .and_then(|t| t.parse().ok())
            .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoData {
    latitude: Option<f64>,
    longitude: Option<f64>,
    altitude: Option<f64>,
    latitude_span: Option<f64>,
    longitude_span: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonParseError(String);
impl<S: ToString> From<S> for JsonParseError {
    fn from(value: S) -> Self {
        Self(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FULL_JSON: &str = r#"
{
  "title": "IMG_0799.HEIC",
  "description": "",
  "imageViews": "30",
  "creationTime": {
    "timestamp": "1563490529",
    "formatted": "Jul 18, 2019, 10:55:29 PM UTC"
  },
  "photoTakenTime": {
    "timestamp": "1563395329",
    "formatted": "Jul 17, 2019, 8:28:49 PM UTC"
  },
  "geoData": {
    "latitude": 10.0,
    "longitude": 92.9,
    "altitude": 100.8,
    "latitudeSpan": 22.2,
    "longitudeSpan": 0.0
  },
  "geoDataExif": {
    "latitude": 0.0,
    "longitude": 0.0,
    "altitude": 0.0,
    "latitudeSpan": 0.0,
    "longitudeSpan": 0.0
  },
  "people": [{
    "name": "Bonnie LaBauve"
  }, {
    "name": "Ryleigh Peterson"
  }],
  "url": "https://lh3.googleusercontent.com/g-Spqub0gJccA3833K32MgbYwk94xS6z47ctcZNTYk3K56yTQdB_vGqio_UnW7XDeIcIU-TEy8uc-iQBMDMqavbTPGLyzCZchzXue8A",
  "googlePhotosOrigin": {
    "mobileUpload": {
      "deviceType": "IOS_PHONE"
    }
  }
}
"#;

    const TEST_EMPTY_JSON: &str = r#"
{
    "title": "IMG_0799.HEIC"
}
"#;

    const TEST_INVALID_JSON: &str = r#"
{
    "title": "IMG_0799.HEIC",
}
"#;

    #[test]
    fn photo_taken_time_is_present() {
        let exif = TakeoutExif::from_json(TEST_FULL_JSON).unwrap();
        assert_eq!(
            exif.photo_taken_time.unwrap().formatted.unwrap(),
            "Jul 17, 2019, 8:28:49 PM UTC"
        );
    }

    #[test]
    fn creation_time_is_present() {
        let exif = TakeoutExif::from_json(TEST_FULL_JSON).unwrap();
        assert_eq!(
            exif.creation_time.unwrap().formatted.unwrap(),
            "Jul 18, 2019, 10:55:29 PM UTC"
        );
    }

    #[test]
    fn geo_data_is_present() {
        let exif = TakeoutExif::from_json(TEST_FULL_JSON).unwrap();
        let geo_data = exif.geo_data.as_ref().unwrap();
        assert_eq!(geo_data.latitude.unwrap(), 10.0);
        assert_eq!(geo_data.longitude.unwrap(), 92.9);
        assert_eq!(geo_data.altitude.unwrap(), 100.8);
        assert_eq!(geo_data.latitude_span.unwrap(), 22.2);
        assert_eq!(geo_data.longitude_span.unwrap(), 0.0);
    }

    #[test]
    fn does_not_fail_when_little_data_is_present() {
        let exif = TakeoutExif::from_json(TEST_EMPTY_JSON).unwrap();
        assert!(exif.creation_time.is_none());
    }

    #[test]
    fn fails_for_invalid_json() {
        let exif = TakeoutExif::from_json(TEST_INVALID_JSON);
        assert!(exif.is_err());
    }
}
