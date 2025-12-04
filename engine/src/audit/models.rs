use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Represents an audit log entry for tracking operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub operation_type: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub old_value: Option<JsonValue>,
    pub new_value: Option<JsonValue>,
    pub initiator: String,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Filter criteria for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditLogFilter {
    pub operation_type: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub initiator: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Pagination parameters for audit log queries
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}
