# winrt-notification

[![license](https://img.shields.io/crates/l/winrt-notification.svg)](https://crates.io/crates/winrt-notification/)
[![version](https://img.shields.io/crates/v/winrt-notification.svg)](https://crates.io/crates/winrt-notification/)
[![Build Status](https://img.shields.io/appveyor/ci/allenbenz/winrt-notification.svg)](https://ci.appveyor.com/project/allenbenz/winrt-notification)

An incomplete wrapper over the WinRT toast api

Tested in windows 10. Untested in Windows 8 and 8.1, might work.

[Documentation](https://allenbenz.github.io/winrt-notification/0_1_3/winrt_notification/)

Todo:
* Add support for Adaptive Content
* Add support for Actions

Known Issues:
* Will not work for Windows 7.
* Will not build when targeting the 32-bit gnu toolchain (i686-pc-windows-gnu).

## Usage

```toml
#Cargo.toml
[dependencies]
winrt-notification = "0.1.4"
```

## Examples

```rust
extern crate winrt_notification;
use winrt_notification::{Duration, Sound, Toast};

fn main() {
    Toast::new("My application name")
        .title("Look at this flip!")
        .text1("(╯°□°）╯︵ ┻━┻")
        .sound(Some(Sound::SMS))
        .duration(Duration::Short)
        .show()
        .expect("unable to toast");
}
```

```rust
extern crate winrt_notification;
use std::path::Path;
use winrt_notification::{IconCrop, Toast};

fn main() {
    Toast::new("application that needs a toast with an image")
        .hero(&Path::new("C:\\absolute\\path\\to\\image.jpeg"), "alt text")
        .icon(
            &Path::new("c:/this/style/works/too/image.png"),
            IconCrop::Circular,
            "alt text",
        )
        .title("Lots of pictures here")
        .text1("One above the text as the hero")
        .text2("One to the left as an icon, and several below")
        .image(&Path::new("c:/photos/sun.png"), "the sun")
        .image(&Path::new("c:/photos/moon.png"), "the moon")
        .sound(None) // will be silent
        .show()
        .expect("unable to toast");
}

```
