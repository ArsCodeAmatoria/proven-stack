//! Export OpenAPI JSON when RUN_OPENAPI_EXPORT=1.

use proven_platform::openapi::ApiDoc;
use std::path::PathBuf;
use utoipa::OpenApi;

#[test]
fn export_openapi_json() {
    if std::env::var("RUN_OPENAPI_EXPORT").ok().as_deref() != Some("1") {
        return;
    }

    let json = ApiDoc::openapi()
        .to_pretty_json()
        .expect("serialize openapi");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = root.join("contracts/openapi/openapi.json");
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    std::fs::write(&out, json).unwrap();
    eprintln!("wrote {}", out.display());
}
