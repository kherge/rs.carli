CarLI
=====

A library for rapid command line tool development.

CarLI is a command line application framework for developing application that provide a single
command and multiple commands. The framework also provides error types tailored for a command
line experience as well as supporting test input and output streams other than those provided
by [`std::io`].

See [`command::Main`] for a complete example.

[`command::Main`]: https://docs.rs/carli/latest/carli/command/trait.Main.html
[`std::io`]: https://doc.rust-lang.org/1.57.0/std/io/index.html

Requirements
------------

- Rust 1.57+

Documentation
-------------

Please see [docs.rs](https://docs.rs/carli/latest/carli).
