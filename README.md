# Embedded graphics extras

A collection of custom primitives and utilities for the [`embedded-graphics`](https://github.com/embedded-graphics/embedded-graphics) library.

## Usage 

###FadedRectangle

A rectangle primitive with directional fade effects. Useful for creating smooth visual transitions on LED displays.

```rust
use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
use embedded_graphics::{FadedRectangle, Fading};

let rect = Rectangle::new(Point::new(0, 0), Size::new(96, 32));
FadedRectangle::new(rect, base_color, Fading::Bottom { steps: 4 })
    .draw(&mut display)?;
```

## Testing

On Mac: 

```bash
export LIBRARY_PATH="$(brew --prefix sdl2)/lib"
cargo test
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
