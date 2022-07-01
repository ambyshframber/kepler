use std::path::{Path, Component};
use chrono::Utc;

pub fn does_path_backtrack(p: impl AsRef<Path>) -> bool {
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
}
pub fn log(s: &str) {
    let now = Utc::now();
    let now_string = now.format("%F %T");
    println!("[{}] {}", now_string, s)
}

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

    #[test]
    fn test_path_backtrack() {
        assert!(!does_path_backtrack("/foo/bar"));
        assert!(!does_path_backtrack("/foo/../bar"));

        assert!(does_path_backtrack("/../foo/bar"));
        assert!(does_path_backtrack("/foo/../../bar"));
        assert!(does_path_backtrack("/foo/././../../../"));
        assert!(does_path_backtrack("/./../"));
    }
}
