use domain::data_export::SinkError;

/// A platform on the allowlist is still not trusted with our memory.
pub(crate) const MAX_BODY_BYTES: usize = 1024 * 1024;

pub(crate) async fn read_capped(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, SinkError> {
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| SinkError::Unreachable(e.without_url().to_string()))?
    {
        if body.len() + chunk.len() > limit {
            return Err(SinkError::Malformed(format!(
                "answer larger than {limit} bytes"
            )));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
