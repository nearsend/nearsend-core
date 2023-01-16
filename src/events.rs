use crate::*;
use near_sdk::serde_json;
use std::fmt;

pub const EVENT_STANDARD_NAME: &str = "nep297"; 
pub const EVENT_VERSION: &str = "1.0.0";

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    UpdateFee(UpdateFeeLog),
    SetOracleId(SetOracleIdLog),
    PayFee(PayFeeLog),
    RefundNear(RefundNearLog),
}

/// Interface to capture data about an event
///
/// Arguments:
/// * `standard`: name of standard e.g. nep297
/// * `version`: e.g. 1.0.0
/// * `event`: associate event data
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,

    // `flatten` to not have "event": {<EventLogVariant>} in the JSON, just have the contents of {<EventLogVariant>}.
    #[serde(flatten)]
    pub event: EventLogVariant,
}

impl fmt::Display for EventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &serde_json::to_string(self).map_err(|_| fmt::Error)?
        ))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdateFeeLog {
    pub old_service_fee: String,
    pub new_service_fee: String,
    pub oracle_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SetOracleIdLog {
    pub old_oracle_id: String,
    pub new_oracle_id: String,
    pub owner_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PayFeeLog {
    pub amount: String,
    pub refund: String,
    pub user_id: String,
    pub old_quota: String,
    pub new_quota: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RefundNearLog {
    pub refund_amount: String,
    pub user_id: String,
}
