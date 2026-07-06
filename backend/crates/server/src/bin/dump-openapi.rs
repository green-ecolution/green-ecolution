use std::env;

use server::http::openapi_doc;

// Dumps the OpenAPI spec for the Green Ecolution backend to stdout.
// The base URL can be overridden via the first CLI argument so production
// dumps embed the deployed origin; falls back to the local dev server otherwise.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:3030".to_string());

    let openapi = openapi_doc(&base_url);
    println!("{}", openapi.to_pretty_json()?);
    Ok(())
}
