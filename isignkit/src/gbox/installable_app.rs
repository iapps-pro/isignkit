use crate::error::InstallableAppError;
use base64::{Engine, prelude::BASE64_STANDARD};
#[cfg(feature = "rocket")]
use rocket::{
    http::uri::{Segments, fmt::Path},
    request::FromSegments,
};
use std::str::FromStr;

#[derive(Debug)]
pub struct InstallableApp {
    pub name: String,
    pub bundle_identifier: String,
    pub version: String,
}

#[cfg(feature = "rocket")]
impl<'r> FromSegments<'r> for InstallableApp {
    type Error = InstallableAppError;

    fn from_segments(segments: Segments<'r, Path>) -> Result<Self, Self::Error> {
        let path = segments
            .to_path_buf(false)
            .map_err(|_| InstallableAppError::InvalidInputString)?;

        let path = path.to_string_lossy();
        Self::from_str(path.as_ref())
    }
}

impl FromStr for InstallableApp {
    type Err = InstallableAppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(prefix) = s.strip_suffix(".plist").filter(|s| !s.is_empty()) else {
            return Err(InstallableAppError::InvalidInputString);
        };

        let decoded = BASE64_STANDARD
            .decode(prefix)
            .map_err(|_| InstallableAppError::InvalidInputString)?;

        let decoded =
            String::from_utf8(decoded).map_err(|_| InstallableAppError::InvalidInputString)?;

        let mut parts = decoded.splitn(3, '/').map(ToString::to_string);

        Ok(InstallableApp {
            name: parts
                .next()
                .ok_or(InstallableAppError::MissingField("name"))?,
            bundle_identifier: parts
                .next()
                .ok_or(InstallableAppError::MissingField("bundleID"))?,
            version: parts
                .next()
                .ok_or(InstallableAppError::MissingField("version"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::InstallableApp;
    #[cfg(feature = "rocket")]
    use rocket::{Rocket, get, http::Status, local::blocking::Client, routes};
    use std::str::FromStr;

    #[cfg(feature = "rocket")]
    #[get("/install/<app_info..>")]
    fn install_app(app_info: InstallableApp) {
        drop(app_info);
    }

    #[test]
    fn parse_invalid() {
        let result = InstallableApp::from_str("install.plist");
        assert!(result.is_err());

        let result = InstallableApp::from_str(".plist");
        assert!(result.is_err());
    }

    #[test]
    fn parse_valid() {
        let path = "aUFwcHMgVGVzdC9ydS5pYXBwcy5UZXN0LUFwcGxpY2F0aW9uLzEuMA==.plist";
        let result = InstallableApp::from_str(path);
        assert!(result.is_ok());
    }

    #[cfg(feature = "rocket")]
    #[test]
    fn rocket_integration() {
        let server = Rocket::build().mount("/", routes![install_app]);
        let client = Client::tracked(server).expect("Can't build client");

        let response = client
            .get("/install/aUFwcHMgVGVzdC9ydS5pYXBwcy5UZXN0LUFwcGxpY2F0aW9uLzEuMA==.plist")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}
