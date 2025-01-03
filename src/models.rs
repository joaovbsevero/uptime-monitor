use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Enum, PartialEq, Eq)]
pub(crate) enum Frequency {
    Hourly,
    Daily,
    Weekly,
}

impl ToString for Frequency {
    fn to_string(&self) -> String {
        match self {
            Frequency::Hourly => "Hourly".to_string(),
            Frequency::Daily => "Daily".to_string(),
            Frequency::Weekly => "Weekly".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Enum, PartialEq, Eq)]
pub(crate) enum HTTPMethod {
    HEAD,
    GET,
}

impl ToString for HTTPMethod {
    fn to_string(&self) -> String {
        match self {
            HTTPMethod::HEAD => "HEAD".to_string(),
            HTTPMethod::GET => "GET".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub(crate) struct Check {
    pub(crate) _id: ObjectId,
    pub(crate) frequency: Frequency,
    pub(crate) url: String,
    pub(crate) method: HTTPMethod,
    pub(crate) expected_body: Option<serde_json::Value>,
    pub(crate) hook: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

impl Check {
    pub(crate) fn from_new(new_check: NewCheck) -> Self {
        Self {
            _id: ObjectId::new(),
            frequency: new_check.frequency,
            url: new_check.url,
            method: new_check.method,
            expected_body: new_check.expected_body,
            hook: new_check.hook,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Enum, PartialEq, Eq)]
pub(crate) enum Status {
    Ok,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub(crate) struct CheckHistory {
    pub(crate) _id: ObjectId,
    pub(crate) check_id: ObjectId,
    pub(crate) status: Status,
    pub(crate) details: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

impl CheckHistory {
    pub(crate) fn new(check_id: ObjectId, status: Status, details: Option<String>) -> Self {
        Self {
            _id: ObjectId::new(),
            check_id,
            status,
            details,
            created_at: Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub(crate) struct NewCheck {
    pub(crate) frequency: Frequency,
    pub(crate) url: String,
    pub(crate) method: HTTPMethod,
    pub(crate) expected_body: Option<serde_json::Value>,
    pub(crate) hook: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub(crate) struct UpdateCheck {
    pub(crate) frequency: Option<Frequency>,
    pub(crate) url: Option<String>,
    pub(crate) method: Option<HTTPMethod>,
    pub(crate) expected_body: Option<Option<serde_json::Value>>,
    pub(crate) hook: Option<Option<String>>,
}

#[derive(Serialize, Deserialize, Clone, Object)]
pub(crate) struct Error {
    detail: String,
    status: u16,
}

impl Error {
    pub(crate) fn bad_request(detail: String) -> Self {
        Self {
            detail,
            status: 400,
        }
    }

    pub(crate) fn not_found(detail: String) -> Self {
        Self {
            detail,
            status: 404,
        }
    }

    pub(crate) fn internal_server_error(detail: String) -> Self {
        Self {
            detail,
            status: 500,
        }
    }
}



#[derive(Serialize, Deserialize)]
pub(crate) struct WebhookData {
    pub(crate) check: Check,
    pub(crate) status: Status,
    pub(crate) details: Option<String>,
}

impl WebhookData {
    pub(crate) fn new(status: Status, details: Option<String>, check: Check) -> Self {
        Self {
            check,
            status,
            details,
        }
    }
}