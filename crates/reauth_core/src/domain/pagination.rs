use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{self, Visitor};
use std::fmt;

#[derive(Debug, Deserialize, Default)]
pub struct PageRequest {
    #[serde(default = "default_page", deserialize_with = "deserialize_i64_from_string")]
    pub page: i64,
    #[serde(default = "default_per_page", deserialize_with = "deserialize_i64_from_string")]
    pub per_page: i64,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default, deserialize_with = "deserialize_sort_dir")]
    pub sort_dir: Option<SortDirection>,
    pub q: Option<String>, // Universal search query
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Asc
    }
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
