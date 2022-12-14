use hyper::{Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;

use crate::config::ReleaseInfoSourceType;
use crate::error::AuError;
use crate::version::ReleaseInfo;

pub struct VersionHandler<'a> {
    uri: &'a str,
}

impl<'a> VersionHandler<'a> {
    /// github release api doc:
    ///
    /// - latests release:
    ///     - GET `/repos/{owner}/{repo}/releases/latest`
    /// - by tag name:
    ///     - GET `/repos/{owner}/{repo}/releases/tags/{tag}`
    ///
    /// the VersionHandler::<TelosGithub> holds value : `/repos/{owner}/{repo}/releases/`,
    /// to support get both latest && some exact tag.
    pub fn new(uri: &'a str, release_info_type: &ReleaseInfoSourceType) -> Self {
        assert!(*release_info_type == ReleaseInfoSourceType::TelosGithub); // only support this for now, make VersionHandler release_info_type generics later
        VersionHandler { uri }
    }

    pub async fn get_release_info(&self, tag_name: Option<String>) -> Result<ReleaseInfo, AuError> {
        let uri = match tag_name {
            Some(tag) => format!("{}/tags/{}", String::from(self.uri), tag),
            None => format!("{}/latest", self.uri),
        };
        let fetch_json = self.get_release_info_json(&uri).await?;
        if let Some(release_info) = ReleaseInfo::new_from_json_object(&fetch_json) {
            Ok(release_info)
        } else {
            Err(AuError::JsonParseError(String::from(
                "release info json parse error",
            )))
        }
    }

    async fn get_release_info_json(&self, uri: &str) -> Result<json::JsonValue, AuError> {
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("User-Agent", "hyper/0.14")
            .header("Accept", "application/vnd.github+json")
            .body(Body::empty())?;
        let https = HttpsConnector::new();
        let resp = Client::builder()
            .build::<_, hyper::Body>(https)
            .request(req)
            .await?;
        if let StatusCode::OK = resp.status() {
            let body_content = hyper::body::to_bytes(resp.into_body()).await?;
            let content = std::str::from_utf8(body_content.as_ref()).unwrap_or("");
            // println!("{}", content);
            return Ok(json::parse(content)?);
        }
        Err(AuError::HttpError(String::from("request error")))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    async fn do_get_release_info() -> Result<ReleaseInfo, AuError> {
        let uri =
            String::from("https://api.github.com/repos/telosprotocol/TOP-Chain/releases/latest");
        let h = VersionHandler::new(&uri, &ReleaseInfoSourceType::TelosGithub);
        h.get_release_info(None).await
    }

    #[test]
    fn test_get_release_info() {
        let r = tokio_test::block_on(do_get_release_info()).unwrap();
        // println!("r: {:?}", r);
        println!("version: {:?}", r.version());
    }
}
