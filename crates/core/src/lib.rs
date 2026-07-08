use serde::{Deserialize, Serialize};

pub mod storage;

/// A Project: the top-level container for Items (see CONTEXT.md).
/// Identified by a unique, filename-safe `slug`; shown to humans by `name`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub slug: Slug,
    pub name: String,
}

/// A validated Project slug: lowercase, non-empty, filename-safe.
/// The inner `String` is private, so the only way to obtain a `Slug`
/// is through `Slug::parse` — a value of this type is always valid.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slug(String);

/// Why a string was rejected as a slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlugError {
    Empty,
    InvalidCharacter(char),
}

impl Slug {
    /// Validate `input` and wrap it as a `Slug`, or explain why it's invalid.
    pub fn parse(input: &str) -> Result<Slug, SlugError> {
        if input.is_empty() {
            return Err(SlugError::Empty);
        }
        for ch in input.chars() {
            let ok = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
            if !ok {
                return Err(SlugError::InvalidCharacter(ch));
            }
        }
        Ok(Slug(input.to_string()))
    }

    /// Borrow the validated slug as a string slice (e.g. for filenames).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_holds_a_valid_slug() {
        let p = Project {
            slug: Slug::parse("tasqs").unwrap(),
            name: String::from("Tasqs"),
        };
        assert_eq!(p.slug.as_str(), "tasqs");
        assert_eq!(p.name, "Tasqs");
    }

    #[test]
    fn parse_accepts_lowercase_alphanumeric_and_hyphens() {
        assert!(Slug::parse("my-project-2").is_ok());
    }

    #[test]
    fn parse_rejects_empty() {
        assert_eq!(Slug::parse(""), Err(SlugError::Empty));
    }

    // YOUR TURN: write a test named `parse_rejects_uppercase` that asserts
    // `Slug::parse("Tasqs")` returns Err(SlugError::InvalidCharacter('T')).
    // Hint: mirror the `parse_rejects_empty` test above.
    #[test]
    fn parse_rejects_uppercase() {
        assert_eq!(Slug::parse("Tasqs"), Err(SlugError::InvalidCharacter('T')));
    }

    #[test]
    fn project_round_trips_through_json() {
        let original = Project {
            slug: Slug::parse("tasqs").unwrap(),
            name: String::from("Tasqs"),
        };
        let json = serde_json::to_string(&original).unwrap();
        // 1. assert the JSON is exactly r#"{"slug":"tasqs","name":"Tasqs"}"#
        //    (this pins down the transparent-slug behavior)
        assert_eq!(json, r#"{"slug":"tasqs","name":"Tasqs"}"#);
        // 2. deserialize with serde_json::from_str::<Project>(&json).unwrap()
        //    and assert it equals `original`
        let deserialized = serde_json::from_str::<Project>(&json).unwrap();
        assert_eq!(deserialized, original);
    }
}
