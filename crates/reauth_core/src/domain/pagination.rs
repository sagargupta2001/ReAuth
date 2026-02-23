use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PageRequest {
    #[serde(
        default = "default_page",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub page: i64,
    #[serde(
        default = "default_per_page",
        deserialize_with = "deserialize_i64_from_string"
    )]
    pub per_page: i64,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default, deserialize_with = "deserialize_sort_dir")]
    pub sort_dir: Option<SortDirection>,
    pub q: Option<String>, // Universal search query
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    20
}

fn deserialize_i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct I64Visitor;

    impl<'de> Visitor<'de> for I64Visitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or a string containing an integer")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            i64::try_from(value).map_err(|_| de::Error::custom("value out of range for i64"))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<i64>()
                .map_err(|_| de::Error::custom("invalid integer string"))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(I64Visitor)
}

fn deserialize_sort_dir<'de, D>(deserializer: D) -> Result<Option<SortDirection>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    let normalized = value.as_deref().map(|v| v.trim().to_lowercase());

    Ok(match normalized.as_deref() {
        Some("asc") => Some(SortDirection::Asc),
        Some("desc") => Some(SortDirection::Desc),
        _ => None,
    })
}

#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Serialize)]
pub struct PageMeta {
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> PageResponse<T> {
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as i64;
        Self {
            data,
            meta: PageMeta {
                total,
                page,
                per_page,
                total_pages,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use super::*;

    #[test]
    fn page_request_defaults_are_applied() {
        let req: PageRequest = serde_json::from_str("{}").expect("default request");
        assert_eq!(req.page, 1);
        assert_eq!(req.per_page, 20);
        assert!(req.sort_by.is_none());
        assert!(req.sort_dir.is_none());
        assert!(req.q.is_none());
    }

    #[test]
    fn page_request_deserializes_string_numbers() {
        let req: PageRequest =
            serde_json::from_str(r#"{"page":"2","per_page":"50"}"#).expect("string numbers");
        assert_eq!(req.page, 2);
        assert_eq!(req.per_page, 50);
    }

    #[test]
    fn page_request_deserializes_numeric_numbers() {
        let req: PageRequest =
            serde_json::from_str(r#"{"page":3,"per_page":10}"#).expect("numeric numbers");
        assert_eq!(req.page, 3);
        assert_eq!(req.per_page, 10);
    }

    #[test]
    fn page_request_deserializes_i64_numbers() {
        let req: PageRequest =
            serde_json::from_str(r#"{"page":-1,"per_page":-5}"#).expect("i64 numbers");
        assert_eq!(req.page, -1);
        assert_eq!(req.per_page, -5);
    }

    #[test]
    fn page_request_rejects_invalid_integer_strings() {
        let err = serde_json::from_str::<PageRequest>(r#"{"page":"abc"}"#)
            .expect_err("invalid integer string should fail");
        let message = err.to_string();
        assert!(
            message.contains("invalid integer string"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn page_request_parses_sort_dir_case_insensitive() {
        let asc: PageRequest = serde_json::from_str(r#"{"sort_dir":"ASC"}"#).expect("asc");
        assert_eq!(asc.sort_dir, Some(SortDirection::Asc));

        let desc: PageRequest = serde_json::from_str(r#"{"sort_dir":" desc "}"#).expect("desc");
        assert_eq!(desc.sort_dir, Some(SortDirection::Desc));

        let none: PageRequest = serde_json::from_str(r#"{"sort_dir":"invalid"}"#).expect("invalid");
        assert_eq!(none.sort_dir, None);
    }

    #[test]
    fn sort_direction_default_is_asc() {
        assert_eq!(SortDirection::default(), SortDirection::Asc);
    }

    #[test]
    fn page_request_rejects_non_numeric_types() {
        let err = serde_json::from_str::<PageRequest>(r#"{"page":true}"#)
            .expect_err("non-numeric should fail");
        let message = err.to_string();
        assert!(
            message.contains("an integer or a string containing an integer"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn deserialize_i64_from_string_handles_owned_string() {
        use serde::de::value::{Error as DeError, I64Deserializer, StringDeserializer};

        let value =
            super::deserialize_i64_from_string(I64Deserializer::<DeError>::new(-42)).expect("i64");
        assert_eq!(value, -42);

        let value = super::deserialize_i64_from_string(StringDeserializer::<DeError>::new(
            "42".to_string(),
        ))
        .expect("string");
        assert_eq!(value, 42);
    }

    #[test]
    fn page_response_computes_total_pages() {
        let response = PageResponse::new(vec!["a"], 21, 1, 20);
        assert_eq!(response.meta.total_pages, 2);

        let empty = PageResponse::<String>::new(Vec::new(), 0, 1, 20);
        assert_eq!(empty.meta.total_pages, 0);
    }
}
