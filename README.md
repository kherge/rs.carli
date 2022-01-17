CarLI
=====

A library for rapid command line tool development.

CarLI is a framework for creating single-command and multi-command CLI applications in Rust. The framework provides error and IO types better suited for the command line environment, especially in cases where unit testing is needed. Opinionated traits are also provided to enforce a consistent way of structuring the application and its subcommands.

See [`command::Main`] for a complete example.

[`command::Main`]: https://docs.rs/carli/latest/carli/command/trait.Main.html

Requirements
------------

- Rust 1.57+

Documentation
-------------

Please see [docs.rs](https://docs.rs/carli/latest/carli).
