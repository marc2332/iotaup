use crate::error::{Error, Result};

/// Validate and normalize a version string. Accepts:
///   1.19.1, v1.19.1, 1.20.0-rc.1, v1.20.0-rc.1
/// Returns the normalized tag with leading `v`.
pub fn normalize(input: &str) -> Result<String> {
    let trimmed = input.trim();
    let body = trimmed.strip_prefix('v').unwrap_or(trimmed);

    let (core, pre) = match body.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (body, None),
    };

    // core: digit.digit.digit
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3
        || !parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(Error::InvalidVersion(input.to_string()));
    }

    if let Some(pre) = pre {
        // accept `rc` or `rc.<digits>`
        if pre == "rc" {
            // ok
        } else if let Some(rest) = pre.strip_prefix("rc.") {
            if rest.is_empty() || !rest.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(Error::InvalidVersion(input.to_string()));
            }
        } else {
            return Err(Error::InvalidVersion(input.to_string()));
        }
    }

    Ok(format!("v{body}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok() {
        assert_eq!(normalize("1.19.1").unwrap(), "v1.19.1");
        assert_eq!(normalize("v1.19.1").unwrap(), "v1.19.1");
        assert_eq!(normalize("1.20.0-rc.1").unwrap(), "v1.20.0-rc.1");
        assert_eq!(normalize("v1.20.0-rc.12").unwrap(), "v1.20.0-rc.12");
        assert_eq!(normalize("1.20.0-rc").unwrap(), "v1.20.0-rc");
    }

    #[test]
    fn bad() {
        assert!(normalize("1.19").is_err());
        assert!(normalize("1.19.1.0").is_err());
        assert!(normalize("1.19.1-beta.1").is_err());
        assert!(normalize("foo").is_err());
        assert!(normalize("1.19.1-rc.").is_err());
    }
}
