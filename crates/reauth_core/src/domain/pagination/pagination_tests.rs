use super::*;

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

    let value =
        super::deserialize_i64_from_string(StringDeserializer::<DeError>::new("42".to_string()))
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
