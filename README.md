# weblum-wasm

This is the code for the Weblum2 app written in Rust and compiled to WASM.

This is a single page app (SPA) for displaying photos and metadata for photos stored in an AWS S3 bucket.

## Developer Instructions

### Prerequisites

Rust compiler setup locally with the Cargo package manager and [Trunk](https://trunkrs.dev/), installed locally.

Build:

`trunk build`

Run locally:

`trunk serve`

`use --release`` flag for minimal wasm bundle size.

### Unit Tests

`cargo test`

`wasm-pack test --chrome`

### Local Browser Configuration

Example Config:

`{"id", "<unique id>": "b": "<bucket name>", "r": "<region>"}`

See `src/local_config.rs` for additional details.

### S3 Configuration

Images will all have the prefix "/images"

Metadata for images will be stored using key "/comments/<e_tag>" where 'e_tag' is the etag of
and Object under the "/images" prefix.