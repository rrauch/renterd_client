# renterd_client

A native Rust client library for the [renterd API](https://api.sia.tech/renterd).

- Fully Async
- Comprehensive (mostly, currently around 90%+)
- Idiomatic types
- Customizable base endpoint
- Rate Limiting (planned)
- Object downloads support AsyncRead & AsyncSeek
- Extensive Sans IO testing

## Overview

`renterd_client` mirrors the [renterd API](https://api.sia.tech/renterd) structure with minor differences.
The three main modules are `autopilot`, `bus` and `worker`. With a fluent API design, all functions are directly
accessible.
For example, to call the `/bus/account/:id/resetdrift` API method you would use the following code:

```rust,no_run
renterd.bus().account().reset_drift(&account_id).await?
```

## Example

This example uses [Tokio](https://tokio.rs), your `Cargo.toml` could look like this:

```toml
[dependencies]
renterd_client = "0.1"
tokio = { version = "1", features = ["full"] }
futures-util = "0.3"
```

And then the code:

```rust,no_run
use futures_util::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // instantiate the client
    let renterd = renterd_client::ClientBuilder::new()
        .api_endpoint_url("http://localhost:9880/api/")
        .api_password("supersecretpassword")
        .build()?;

    // dismiss all alerts
    renterd.bus().alert().dismiss(None).await?;

    // get the current state of the autopilot
    let state = renterd.autopilot().state().await?;

    // download a file
    if let Some(file) = renterd
        .worker()
        .object()
        .download("/foo/bar/file.txt", None)
        .await?
    {
        let mut stream = file.open_stream().await?;
        let mut content = Vec::with_capacity(file.length.unwrap() as usize);
        stream.read_to_end(&mut content).await?;
    }

    Ok(())
}
```

## Status

It's still very early days. There is a large number of unit tests covering most functions, but given the sheer number of
functions that the [renterd API](https://api.sia.tech/renterd) exposes, some issues are to be expected. Use it at your
own risk.

Contributions are certainly welcome :smile:

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
