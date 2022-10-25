<div align="center">

<a href="https://crates.io/crates/certain">
<h2><b>Certain</b></h2>
</a>

__Certificate Transparency Log Utility__

[![dependency status](https://deps.rs/crate/certain/0.1.0/status.svg)](https://deps.rs/crate/certain/0.1.0)
[![Documentation](https://docs.rs/certein/badge.svg)](https://docs.rs/certain)
[![License](https://img.shields.io/crates/l/certain.svg)](https://choosealicense.com/licenses/mit/)

</div>

---

Lightweight utility for listening to certificate transparency logs.

## Usage
To use `certain`, add this to your `Cargo.toml`:

```toml
[dependencies]
certain = "0.1.0"
```

## Example
The following example will stream the latest certificates appended to the log.

```rust
use std::time::{Duration};

use certain::{
    
    StreamConfig,
    StreamError, 
};
 
fn main() -> Result<(), StreamError> {
    let config = StreamConfig::new("https://ct.googleapis.com/logs/argon2022/")
        .timeout(Duration::from_secs(1));

    certain::stream(config, |entry| {
        println!("{entry:#?}");
        true
    })?;

    Ok(())
}
```

## Contributing
All contributions are welcome, don't hesitate to open an issue if something is missing!

## License
[MIT](https://choosealicense.com/licenses/mit/)