use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FearGreedResponse {
    pub data: Vec<FearGreedData>,
    pub status: FearGreedStatus,
}

#[derive(Debug, Deserialize)]
pub struct FearGreedStatus {
    #[serde(rename = "error_code")]
    pub error_code_str: String,
    #[serde(rename = "error_message")]
    pub error_message: String,
}

#[derive(Debug, Deserialize)]
pub struct FearGreedData {
    pub timestamp: String,
    pub value: u64,
    pub value_classification: String,
}
