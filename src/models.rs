#[derive(serde::Deserialize, Clone)]
pub struct Measure {
    pub district_id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}