use std::str::FromStr;

use actix_web::http::Uri;
use log::{debug, error};

use crate::files::FileID;
use crate::state::AppState;

/// WARNING: DO NOT PUT STUFF INSIDE THE BRACKETS LIKE SIMPLE FORMAT SYNTAX. THESE ITEMS WILL NOT
/// BE URLENCODED!
///
/// DO: `uri_any!("/foo/{}", 1337)`
///
/// DO NOT: `uri_any!("/foo/{1337}")`
macro_rules! uri_any {
    ($base:literal $(,$i:expr)*) => {
        format!($base $(, urlencoding::encode($i.to_string().as_str()))*)
    };
}

impl AppState<'_> {
    pub fn uri_api_file_fid(&self, fid: FileID) -> Uri {
        self.uri_any(&uri_any!("/api/v1/file/{}", fid))
    }

    pub fn uri_api_file_fid_name(&self, fid: FileID, name: &str) -> Uri {
        self.uri_any(&uri_any!("/api/v1/file/{}/{}", fid, name))
    }

    pub fn base_uri(&self) -> Uri {
        Uri::from_str(&self.config.service.base_url)
            .expect("base_url of config was not a proper url")
    }

    pub fn uri_api_file_fid_name_info(&self, fid: FileID, name: &str) -> Uri {
        self.uri_any(&uri_any!("/api/v1/file/{}/{}/info", fid, name))
    }

    pub fn uri_frontend_file(&self) -> Uri {
        self.uri_any(&uri_any!("/file"))
    }

    pub fn uri_frontend_file_fid(&self, fid: FileID) -> Uri {
        self.uri_any(&uri_any!("/file/{}", fid))
    }

    pub fn uri_frontend_index(&self) -> Uri {
        self.uri_any(&uri_any!("/"))
    }

    fn uri_any(&self, raw_url: &str) -> Uri {
        let u = self.base_uri();
        let mut parts = u.into_parts();
        parts.path_and_query = Some(
            raw_url
                .parse()
                .inspect_err(|e| error!("Made a faulty URI '{}' somehow: {e}", raw_url))
                .expect("could not format url for fid"),
        );

        Uri::from_parts(parts).expect("could not combine uri parts for url")
    }
}
