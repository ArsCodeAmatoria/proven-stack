//! Redoc UI for OpenAPI (foundation docs surface).

use axum::response::Html;

/// `GET /redoc` — Redoc served from CDN against `/api-docs/openapi.json`.
pub async fn redoc() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Proven API — Redoc</title>
    <style>
      body { margin: 0; padding: 0; }
    </style>
  </head>
  <body>
    <redoc spec-url="/api-docs/openapi.json"></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
  </body>
</html>"#,
    )
}
