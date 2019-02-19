/*!
[![Build Status](https://travis-ci.org/bcmyers/num-format.svg?branch=master)](https://travis-ci.org/bcmyers/num-format)
[![Crates.io](https://img.shields.io/crates/v/num-format.svg)](https://crates.io/crates/num-format)
[![Documentation](https://docs.rs/num-format/badge.svg)](https://docs.rs/num-format/)
![License](https://img.shields.io/crates/l/num_format.svg)

A Rust crate for producing string representations of numbers, formatted according to international
standards, e.g.

* `"1,000,000"` for US English
* `"10,00,000"` for Indian English
* `"1 000 000"` for French French

# Creating a string representation

**num-format** offers **three** principal APIs...

### `ToFormattedString`

The [`ToFormattedString`] trait is the simplist of the three APIs. Just call
[`to_formatted_string`] on a type that implements it (all the integer types in the standard library
implement it) while providing a desired format (see [picking a format] below). That said, using
[`ToFormattedString`] will always heap allocate; so it is the slowest of the three APIs and cannot
be used in a `no_std` environment.

```rust
# use cfg_if::cfg_if; cfg_if! { if #[cfg(feature = "std")] {
use num_format::{Locale, ToFormattedString};

fn main() {
    let s = 1000000.to_formatted_string(&Locale::en);
    assert_eq!(&s, "1,000,000");
}
# } else { fn main() {} } }
```

### `Buffer`

Using the [`Buffer`] type is the fastest API, as it does **not** heap allocate. Instead, the
formatted representation is written into a stack-allocated buffer. As such, you can use it in a
`no_std` environment.

Although this API is available for all the integer types in the standard library, it is **not**
available for types like [`num_bigint::BigInt`] whose maximum size cannot be known in advance.

```rust
use num_format::{Buffer, Locale};

fn main() {
    // Create a stack-allocated buffer...
    let mut buf = Buffer::default();

    // Write "1,000,000" into the buffer...
    buf.write_formatted(&1000000, &Locale::en);

    // Get a view into the buffer as a &str...
    let s = buf.as_str();

    // Do what you want with the &str...
    assert_eq!("1,000,000", s);
}
```

### `WriteFormatted`

The [`WriteFormatted`] trait is in between the other two APIs. You can write a formatted
representation into any type that implements [`WriteFormatted`] (all the types in the standard
library that implement [`io::Write`] or [`fmt::Write`] implement [`WriteFormatted`], such as
[`Vec`], [`String`], [`File`], etc.).

If you're writing a number type that can use the [`Buffer`] API, there is **no** heap allocation.
That said, the [`io::Write`] and [`fmt::Write`] machinery adds a bit of overhead; so it's faster
to use the [`Buffer`] type directly.

You can also use this API with types where the [`Buffer`] API will not work, like
[`num_bigint::BigInt`], in which case there will be heap allocation. As such, this trait is
**not** available in a `no_std` environment.

```rust
# use cfg_if::cfg_if; cfg_if! { if #[cfg(feature = "std")] {
use num_format::{Locale, WriteFormatted};

fn main() {
    // Create a writer...
    let mut writer = String::new(); // Could also be Vec::new(), File::open(...), ...

    // Write "1,000,000" into the writer...
    writer.write_formatted(&1000000, &Locale::en);

    assert_eq!(&writer, "1,000,000");
}
# } else { fn main() {} } }
```

# Picking a format

Formatting options (e.g. which thousands separator to use, what the minus sign looks like, etc.)
are represented by the [`Format`] trait. This crate offers **three** concrete implementations of
the [`Format`] trait...

### `Locale`

The [`Locale`] type is a programatically generated enum representing formatting standards from the
[Common Locale Data Repository], which is maintained by the [Unicode Consortium] and used by
Apple in macOS and iOS, by LibreOffice, by IBM in AIX, among others.

```rust
use num_format::{Grouping, Locale};

fn main() {
    let locale = Locale::en;
    assert_eq!(locale.grouping(), Grouping::Standard);
    assert_eq!(locale.minus_sign(), "-");
    assert_eq!(locale.name(), "en");
    assert_eq!(locale.separator(), ",");

    let locale2 = Locale::from_name("en").unwrap();
    assert_eq!(locale, locale2);

    let available = Locale::available_names();
    println!("All of the locale names available in the Unicode database are...");
    println!("{:#?}", available);
}
```

### `SystemLocale`

The [`SystemLocale`] type is another type that implements [`Format`]. It allows you to access your
OS's locale information. It has a very similar API to [`Locale`] and should work on all major
operating systems (i.e. macOS, linux, the BSDs, and Windows).

* On Unix-based systems, the underlying implementation uses [`locale`], [`newlocale`],
  [`uselocale`] and other POSIX functions from the C standard library.
* On Windows, the [`GetLocaleInfoEx`] and [`EnumSystemLocalesEx`] functions from the Windows API
  are used.

```rust
# #[cfg(all(feature = "std", any(unix, windows)))]
use num_format::SystemLocale;

# #[cfg(all(feature = "std", any(unix, windows)))]
fn main() {
    let locale = SystemLocale::default().unwrap();
    println!("My system's default locale is...");
    println!("{:#?}", &locale);

    let available = SystemLocale::available_names().unwrap();
    println!("My available locale names are...");
    println!("{:#?}", available);

    match SystemLocale::from_name("en_US") {
        Ok(_) => println!("My system has the 'en_US' locale."),
        Err(_) => println!("The 'en_US' locale is not included with my system."),
    }
}
# #[cfg(not(all(feature = "std", any(unix, windows))))]
# fn main() {}
```

### `CustomFormat`

[`CustomFormat`] is the third and final type that implements [`Format`]. You can use it to build
your own custom formats.

```rust
use num_format::{Buffer, Error, CustomFormat, Grouping};

fn main() -> Result<(), Error> {
    let format = CustomFormat::builder()
        .grouping(Grouping::Indian)
        .minus_sign("🙌")
        .separator("😀")
        .build()?;

    let mut buf = Buffer::new();
    buf.write_formatted(&(-1000000), &format);
    assert_eq!("🙌10😀00😀000", buf.as_str());

    Ok(())
}
```

# Extra features

| Available features | What to put in your `Cargo.toml`                              |
| :----------------- | :------------------------------------------------------------ |
| `no_std`           | `num-format = { version = "0.3", default-features = false }`  |
| `num-bigint`       | `num-format = { version = "0.3", features = ["num-bigint"] }` |
| `serde`            | `num-format = { version = "0.3", features = ["with-serde"] }` |

# License

**num-format** is licensed under either of:

- [The Apache License, Version 2.0], or
- [The MIT license]

at your option.

[`Buffer`]: struct.Buffer.html
[Common Locale Data Repository]: https://en.wikipedia.org/wiki/Common_Locale_Data_Repository
[`CustomFormat`]: format/struct.CustomFormat.html
[`EnumSystemLocalesEx`]: https://docs.microsoft.com/en-us/windows/desktop/api/winnls/nf-winnls-enumsystemlocalesex
[`File`]: https://doc.rust-lang.org/std/fs/struct.File.html
[`fmt::Write`]: https://doc.rust-lang.org/std/fmt/fn.write.html
[`Format`]: format/trait.Format.html
[`GetLocaleInfoEx`]: https://docs.microsoft.com/en-us/windows/desktop/api/winnls/nf-winnls-getlocaleinfoex
[`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
[`locale`]: http://man7.org/linux/man-pages/man1/locale.1.html
[`Locale`]: format/enum.Locale.html
[`localeconv`]: https://www.gnu.org/software/libc/manual/html_node/The-Lame-Way-to-Locale-Data.html#The-Lame-Way-to-Locale-Data
[`newlocale`]: http://man7.org/linux/man-pages/man3/newlocale.3.html
[`num_bigint::BigInt`]: https://docs.rs/num-bigint/0.2.2/num_bigint/struct.BigInt.html
[picking a format]: #picking-a-format
[`setlocale`]: https://www.gnu.org/software/libc/manual/html_node/Setting-the-Locale.html
[`String`]: https://doc.rust-lang.org/std/string/struct.String.html
[`SystemLocale`]: format/struct.SystemLocale.html
[The Apache License, Version 2.0]: http://www.apache.org/licenses/LICENSE-2.0
[The MIT license]: http://opensource.org/licenses/MIT
[`ToFormattedString`]: trait.ToFormattedString.html
[`to_formatted_string`]: trait.ToFormattedString.html#method.to_formatted_string
[Unicode Consortium]: https://en.wikipedia.org/wiki/Unicode_Consortium
[`uselocale`]: http://man7.org/linux/man-pages/man3/uselocale.3.html
[`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
[`WriteFormatted`]: trait.WriteFormatted.html
*/

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    dead_code,
    deprecated,
    // missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_imports,
    unused_macros,
    unused_mut,
    unused_results,
    unused_parens,
    unused_unsafe,
    unused_variables
)]
#![doc(html_root_url = "https://docs.rs/num-format/0.3.0")]

#[cfg(all(feature = "std", unix))]
#[macro_use]
extern crate cfg_if;

#[cfg(all(feature = "std", any(unix, windows)))]
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde;

mod buffer;
mod constants;
mod custom_format;
mod custom_format_builder;
mod error;
mod error_kind;
mod format;
mod grouping;
mod impls;
mod locale;
mod strings;
#[cfg(all(feature = "std", any(unix, windows)))]
mod system_locale;
mod to_formatted_str;
#[cfg(feature = "std")]
mod to_formatted_string;
#[cfg(feature = "std")]
mod write_formatted;

pub use self::buffer::Buffer;
pub use self::custom_format::CustomFormat;
pub use self::custom_format_builder::CustomFormatBuilder;
pub use self::error::Error;
pub use self::error_kind::ErrorKind;
pub use self::format::Format;
pub use self::grouping::Grouping;
pub use self::locale::Locale;
#[cfg(all(feature = "std", any(unix, windows)))]
pub use self::system_locale::SystemLocale;
pub use self::to_formatted_str::ToFormattedStr;
#[cfg(feature = "std")]
pub use self::to_formatted_string::ToFormattedString;
#[cfg(feature = "std")]
pub use self::write_formatted::WriteFormatted;

mod sealed {
    pub trait Sealed {}
}

pub mod utils {
    //! Utility types needed if you want to implement [`Format`] on your own type.
    //!
    //! [`Format`]: trait.Format.html

    pub use crate::strings::{
        DecimalStr, InfinityStr, MinusSignStr, NanStr, PlusSignStr, SeparatorStr,
    };
}
