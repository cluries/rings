#[inline]
pub fn api_v1(suffix: &str) -> String {
    format!("/api/v1/{}", suffix)
}
