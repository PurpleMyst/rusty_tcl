rusty-tcl
=========

`rusty-tcl` is a rustic binding to TCL which allows you to embed the much-loved
scripting language into your rust programs.

Until we reach 1.0.0, this library is *very* unstable. Breaking changes happen
pretty much every version.

Links
-----

- [Documentation](https://purplemyst.github.io/rusty_tcl)
- [crates.io page](https://crates.io/crates/rusty-tcl)
- [patreon page](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

Requirements
------------

1. `clang`, preferably the newest version you can get, but at least `>=4.0`.  
I won't go into details on how to install this as it should be pretty straight forward.

2. `tcl8.5` and its developer headers, you can install this from most
package managers. On a Debian-based operating system such as Ubuntu or
Debian itself, you can run `sudo apt install tcl8.5 tcl8.5-dev`.

3. Rust. You can install this by following [the installation guide on the
official rust website](https://www.rust-lang.org/en-US/install.html).

Compilation
-----------

First off, install all the requirements. Afterwards, you should run `updatedb`.
This is because the custom build script for `rusty-tcl-sys` utilizes `locate`
to find your TCL installation. While in a perfect world it shouldn't, I found
it's the easiest way to do it.

Then, a simple `cargo build --release` will build the package. You can run a
few tests via `cargo test`.

If you need to use this as a library in your project, you can just put the
following into your `Cargo.toml`, after the `[dependencies]` header:

```toml
rusty-tcl = "*"
```

License
-------

This package is MIT-licensed.
