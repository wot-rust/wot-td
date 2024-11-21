//! CoAP Binding Template

use crate::extend::ExtendableThing;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, skip_serializing_none};

/// CoAP request method
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Patch,
    Fetch,
    #[serde(rename = "iPATCH")]
    Ipatch,
}

/// CoAP Allowed block size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
pub enum BlockSize {
    Size16 = 16,
    Size32 = 32,
    Size64 = 64,
    Size128 = 128,
    Size256 = 256,
    Size512 = 512,
    Size1024 = 1024,
}

/// CoAP BlockWise Transfer Parameters
///
/// They may apply to Block-Wise Transfers [RFC7959] or
/// Block-Wise Transfer Options Supporting Robust Transmission [RFC9177].
///
/// [RFC7959]: https://www.rfc-editor.org/rfc/rfc7959.html
/// [RFC9177]: https://www.rfc-editor.org/rfc/rfc9177.html
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct BlockWiseTransferParameters {
    #[serde(rename = "cov:block2Size")]
    pub block2_size: Option<BlockSize>,
    #[serde(rename = "cov:block1Size")]
    pub block1_size: Option<BlockSize>,
}

/// CoAP Protocol Form fields
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct Form {
    #[serde(rename = "cov:method")]
    pub method: Option<Method>,
    #[serde(rename = "cov:blockwise")]
    pub blockwise: Option<BlockWiseTransferParameters>,
    #[serde(rename = "cov:qblockwise")]
    pub qblockwise: Option<BlockWiseTransferParameters>,
    #[serde(rename = "cov:hopLimit")]
    pub hop_limit: Option<u8>,
    #[serde(rename = "cov:accept")]
    pub accept: Option<u16>,
    #[serde(rename = "cov:contentFormat")]
    pub content_format: Option<u16>,
}

/// CoAP Protocol ExpectedResponse fields
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct ExpectedResponse {
    #[serde(rename = "cov:contentFormat")]
    pub content_format: Option<u16>,
}

/// Extension for the CoAP protocol
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct CoapProtocol {}

impl ExtendableThing for CoapProtocol {
    type InteractionAffordance = ();
    type PropertyAffordance = ();
    type ActionAffordance = ();
    type EventAffordance = ();
    type Form = Form;
    type ExpectedResponse = ExpectedResponse;
    type DataSchema = ();
    type ObjectSchema = ();
    type ArraySchema = ();
}

#[cfg(test)]
mod test {
    use alloc::vec;

    use super::{BlockSize, CoapProtocol};
    use crate::thing::{ExpectedResponse, Form};
    fn deserialize_form(s: &str, r: Form<CoapProtocol>) {
        let f: crate::thing::Form<CoapProtocol> = serde_json::from_str(s).unwrap();

        assert_eq!(f, r);
    }

    #[test]
    fn deserialize_observe() {
        let form = r#"
            {
                "cov:method": "GET",
                "href": "coap://[2001:DB8::1]/status",
                "contentType": "text/plain;charset=utf-8",
                "subprotocol": "cov:observe",
                "op": ["observeproperty"]
            }
        "#;
        let expected = Form {
            op: crate::thing::DefaultedFormOperations::Custom(vec![
                crate::thing::FormOperation::ObserveProperty,
            ]),
            href: "coap://[2001:DB8::1]/status".into(),
            content_type: Some("text/plain;charset=utf-8".into()),
            subprotocol: Some("cov:observe".into()),
            other: super::Form {
                method: Some(super::Method::Get),
                ..Default::default()
            },
            ..Default::default()
        };

        deserialize_form(form, expected);
    }

    #[test]
    fn deserialize_blockwise() {
        let form = r#"
            {
                "href": "coap://[2001:DB8::1]/status",
                "contentType": "text/plain;charset=utf-8",
                "cov:blockwise": { }
            }
        "#;
        let expected = Form {
            href: "coap://[2001:DB8::1]/status".into(),
            content_type: Some("text/plain;charset=utf-8".into()),
            other: super::Form {
                blockwise: Some(super::BlockWiseTransferParameters::default()),
                ..Default::default()
            },
            ..Default::default()
        };

        deserialize_form(form, expected);
    }

    #[test]
    fn deserialize_qblockwise_params() {
        let form = r#"
            {
                "href": "coap://[2001:DB8::1]/status",
                "contentType": "text/plain;charset=utf-8",
                "cov:qblockwise": {
                    "cov:block2Size": 64
                }
            }
        "#;
        let expected = Form {
            href: "coap://[2001:DB8::1]/status".into(),
            content_type: Some("text/plain;charset=utf-8".into()),
            other: super::Form {
                qblockwise: Some(super::BlockWiseTransferParameters {
                    block2_size: Some(BlockSize::Size64),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        deserialize_form(form, expected);
    }

    #[test]
    fn deserialize_hop_limit() {
        let form = r#"
            {
                "href": "coap://[2001:DB8::1]/status",
                "contentType": "text/plain;charset=utf-8",
                "cov:hopLimit": 5
            }
        "#;
        let expected = Form {
            href: "coap://[2001:DB8::1]/status".into(),
            content_type: Some("text/plain;charset=utf-8".into()),
            other: super::Form {
                hop_limit: Some(5),
                ..Default::default()
            },
            ..Default::default()
        };

        deserialize_form(form, expected);
    }

    #[test]
    fn deserialize_content_format() {
        let form = r#"
            {
                "href": "coap://[2001:DB8::1]/status",
                "contentType": "application/cbor",
                "cov:contentFormat": 60,
                "cov:accept": 60,
                "response": {
                    "contentType": "application/cbor",
                    "cov:contentFormat": 60
                }
            }
        "#;
        let expected = Form {
            href: "coap://[2001:DB8::1]/status".into(),
            content_type: Some("application/cbor".into()),
            other: super::Form {
                content_format: Some(60),
                accept: Some(60),
                ..Default::default()
            },
            response: Some(ExpectedResponse {
                content_type: "application/cbor".into(),
                other: super::ExpectedResponse {
                    content_format: Some(60),
                },
            }),
            ..Default::default()
        };

        deserialize_form(form, expected);
    }
}
