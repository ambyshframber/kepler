use std::path::{Path, PathBuf, Component};
//use chrono::Utc;

/*pub fn does_path_backtrack(p: impl AsRef<Path>) -> bool {
    let p = p.as_ref();
    let mut steps_forward = 0; // count how many times we go into or out of a directory
    for segment in p.components() {
        if let Component::Normal(_) = segment {
            steps_forward += 1
        }
        else if segment == Component::ParentDir {
            steps_forward -= 1
        }

        if steps_forward < 0 { // if you ever leave the "root", immediately reject
            return true
        }
    }

    false
}*/
pub fn normalise_path(p: impl AsRef<Path>) -> PathBuf {
    let p = p.as_ref();
    let mut ret = PathBuf::new();
    for segment in p.components() {
        match segment {
            Component::Normal(c) => ret.push(c),
            Component::ParentDir => { ret.pop(); }
            _ => {}
        }
    }
    ret
}
const GEMINI_SCHEME: &str = "gemini://";
fn is_uri_gemini(uri: &str) -> Option<&str> {
    if uri.starts_with(GEMINI_SCHEME) {
        Some(&uri[GEMINI_SCHEME.len()..])
    }
    else {
        None
    }
}
#[derive(Debug, PartialEq)]
pub struct Uri<'a> {
    pub hostname: &'a str,
    pub path: &'a str,
    pub query: &'a str
}
impl Uri<'_> {
    fn _new(uri: &str) -> Option<Uri> {
        let uri = is_uri_gemini(uri.trim())?;
        let first_slash = uri.find('/');
        match first_slash {
            Some(i) => {
                let hostname = &uri[..i];
                let rest = &uri[i..];
                let (path, query) = rest.split_once('?').unwrap_or((rest, ""));
                Some(Uri {
                    hostname, path, query
                })
            }
            None => {
                Some(Uri {
                    hostname: uri,
                    path: "/",
                    query: ""
                })
            }
        }
    }
    pub fn new(uri: &str) -> Result<Uri, GeminiError> {
        Self::_new(uri).ok_or(GeminiError::bad_request(""))
    }
}

/*pub fn log(s: &str) {
    let now = Utc::now();
    let now_string = now.format("%F %T");
    println!("[{}] {}", now_string, s)
}*/

#[derive(Debug, PartialEq)]
pub struct GeminiError {
    code: u8,
    meta: String
}
impl GeminiError {
    pub fn to_string(&self) -> String {
        format!("{} {}\r\n", self.code, self.meta)
    }

    pub fn bad_request(meta: &str) -> GeminiError {
        GeminiError {
            code: BAD_REQUEST,
            meta: meta.into()
        }
    }
    pub fn not_found() -> GeminiError {
        GeminiError {
            code: NOT_FOUND,
            meta: String::new()
        }
    }
    pub fn redirect(new_url: &str, permanent: bool) -> GeminiError {
        GeminiError {
            code: redirect(permanent),
            meta: new_url.into()
        }
    }
    pub fn temporary_failure(meta: &str) -> GeminiError {
        GeminiError {
            code: TEMP_FAIL,
            meta: meta.into()
        }
    }
}

fn redirect(permanent: bool) -> u8 {
    if permanent {
        31
    }
    else {
        30
    }
}
const TEMP_FAIL: u8 = 40;
const NOT_FOUND: u8 = 51;
const BAD_REQUEST: u8 = 59;

#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn test_path_backtrack() {
        assert!(!does_path_backtrack("/foo/bar"));
        assert!(!does_path_backtrack("/foo/../bar"));

        assert!(does_path_backtrack("/../foo/bar"));
        assert!(does_path_backtrack("/foo/../../bar"));
        assert!(does_path_backtrack("/foo/././../../../"));
        assert!(does_path_backtrack("/./../"));
    }*/

    #[test]
    fn normalise() {
        assert_eq!(normalise_path("a/b/c"), PathBuf::from("a/b/c"));
        assert_eq!(normalise_path("a/b/c/../d"), PathBuf::from("a/b/d"));
        assert_eq!(normalise_path("/../b/c"), PathBuf::from("b/c"));
        assert_eq!(normalise_path("/a/b/c/../d"), PathBuf::from("a/b/d"));
        assert_eq!(normalise_path("/../foo/bar/../../a"), PathBuf::from("a"));
        assert_eq!(normalise_path("a/b/c/"), PathBuf::from("a/b/c"));
    }

    #[test]
    fn uri_parse() {
        assert_eq!(
            Uri::new("gemini://example.com"),
            Ok(Uri {
                hostname: "example.com",
                path: "/",
                query: ""
            })
        );
        assert_eq!(
            Uri::new("gemini://example.com/path/to/content"),
            Ok(Uri {
                hostname: "example.com",
                path: "/path/to/content",
                query: ""
            })
        );
        assert_eq!(
            Uri::new("gemini://example.com/path?query"),
            Ok(Uri {
                hostname: "example.com",
                path: "/path",
                query: "query"
            })
        );
    }
}
