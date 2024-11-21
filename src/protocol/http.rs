//! HTTP Binding Template

use alloc::{string::String, vec::Vec};

use crate::extend::ExtendableThing;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

/// HTTP request method
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Patch,
}

/// HTTP Header
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct MessageHeader {
    #[serde(rename = "htv:fieldName")]
    pub field_name: Option<String>,
    #[serde(rename = "htv:fieldValue")]
    pub field_value: Option<String>,
}

/// Extended fields for ExpectedResponse and AdditionalResponse
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, Hash)]
pub struct Response {
    #[serde(rename = "htv:headers")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<MessageHeader>,
    #[serde(rename = "htv:statusCodeValue")]
    pub status_code_value: Option<usize>,
}

/// Extended fields for Form
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq, Hash)]
pub struct Form {
    #[serde(rename = "htv:methodName")]
    pub method_name: Option<Method>,
}

/// HTTP Protocol extension
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct HttpProtocol {}

impl ExtendableThing for HttpProtocol {
    type InteractionAffordance = ();
    type PropertyAffordance = ();
    type ActionAffordance = ();
    type EventAffordance = ();
    type Form = Form;
    type ExpectedResponse = Response;
    type DataSchema = ();
    type ObjectSchema = ();
    type ArraySchema = ();
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::HttpProtocol;
    use crate::thing::{ExpectedResponse, Form};

    fn deserialize_form(s: &str, r: Form<HttpProtocol>) {
        let f: Form<HttpProtocol> = serde_json::from_str(s).unwrap();
        assert_eq!(f, r);
    }

    #[test]
    fn deserialize_discovery_property() {
        let property = r#"
        {
            "href": "/things{?offset,limit,format,sort_by,sort_order}",
            "htv:methodName": "GET",
            "response": {
                "description": "Success response",
                "htv:statusCodeValue": 200,
                "contentType": "application/ld+json",
                "htv:headers": [
                    {
                        "htv:fieldName": "Link"
                    }
                ]
            }
        }
        "#;

        let expected = Form {
            href: "/things{?offset,limit,format,sort_by,sort_order}".into(),
            response: Some(ExpectedResponse {
                content_type: "application/ld+json".into(),
                other: super::Response {
                    headers: vec![super::MessageHeader {
                        field_name: Some("Link".into()),
                        field_value: None,
                    }],
                    status_code_value: Some(200),
                },
            }),
            other: super::Form {
                method_name: Some(super::Method::Get),
            },
            ..Default::default()
        };

        deserialize_form(property, expected);
    }

    #[test]
    fn deserialize_discovery_action() {
        let action = r#"
        {
            "href": "/things",
            "htv:methodName": "POST",
            "response": {
                "contentType": "application/td+json",
                "description": "Success response including the system-generated URI",
                "htv:headers": [
                    {
                        "description": "System-generated URI",
                        "htv:fieldName": "Location"
                    }
                ],
                "htv:statusCodeValue": 201
            }
        }
        "#;

        let expected = Form {
            op: Default::default(),
            href: "/things".into(),
            response: Some(ExpectedResponse {
                content_type: "application/td+json".into(),
                other: super::Response {
                    headers: vec![super::MessageHeader {
                        field_name: Some("Location".into()),
                        field_value: None,
                    }],
                    status_code_value: Some(201),
                },
            }),
            other: super::Form {
                method_name: Some(super::Method::Post),
            },
            ..Default::default()
        };

        deserialize_form(action, expected);
    }
}
