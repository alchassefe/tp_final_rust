use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "UPPERCASE")]
pub enum Command {
    Ping,
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    Del {
        key: String,
    },
    Keys,
    Expire {
        key: String,
        seconds: u64,
    },
    Ttl {
        key: String,
    },
    Incr {
        key: String,
    },
    Decr {
        key: String,
    },
    Save,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize)]
pub struct Response {
    pub status: String,
    pub value: Option<serde_json::Value>,
    pub count: Option<usize>,
    pub keys: Option<Vec<String>>,
    pub ttl: Option<i64>,
    pub message: Option<String>,
}

impl Response {
    pub fn ok() -> Self {
        Self {
            status: "ok".into(),
            value: None,
            count: None,
            keys: None,
            ttl: None,
            message: None,
        }
    }
    pub fn error(msg: &str) -> Self {
        Self {
            status: "error".into(),
            value: None,
            count: None,
            keys: None,
            ttl: None,
            message: Some(msg.into()),
        }
    }
}
