//! Provides commands with an abstraction over input and output stream.
//!
//! Typically, commands will use the standard input and output streams provided by the operating
//! environment. While convenient, especially when coupled with Rust standard library features such
//! as [`std::io::Stdout`], we may not always want to use them. For example, there may be cases when
//! you want to test what is being written to or read from these streams. To support those cases, a
//! layer of abstraction is introduced.
//!
//! ```
//! use carli::error::Result;
//! use carli::io::{Context, Standard};
//! use carli::outputln;
//!
//! fn example(context: &impl Context) -> Result<()> {
//!     outputln!(context, "Hello, world!")?;
//!
//!     Ok(())
//! }
//!
//! fn main() {
//!     let context = Standard::default();
//!
//!     if let Err(error) = example(&context) {
//!         error.exit();
//!     }
//! }
//! ```
//!
//! ### Testing command input
//!
//! ```
//! use carli::error::Result;
//! use carli::io::{Context, Test};
//! use carli::outputln;
//! use std::io::{BufReader, Read};
//!
//! fn example(context: &impl Context) -> Result<()> {
//!     let input: Result<Vec<u8>> = context.input(|s| {
//!         let mut buffer = Vec::new();
//!         let mut reader = BufReader::new(s);
//!
//!         reader.read_to_end(&mut buffer)?;
//!
//!         Ok(buffer)
//!     });
//!
//!     if let Ok(content) = input {
//!         let name = String::from_utf8_lossy(content.as_slice());
//!
//!         outputln!(context, "Hello, {}!", name);
//!     }
//!
//!     Ok(())
//! }
//!
//! fn main() {
//!     let context = Test::default();
//!
//!     context.set_input(b"example");
//!
//!     example(&context).unwrap();
//! }
//! ```
//!
//! ### Testing command output
//!
//! ```
//! use carli::error::Result;
//! use carli::io::{Context, Test};
//! use carli::outputln;
//!
//! fn example(context: &impl Context) -> Result<()> {
//!     outputln!(context, "Hello, world!")?;
//!
//!     Ok(())
//! }
//!
//! #[cfg(test)]
//! mod test {
//!     use super::*;
//!
//!     fn example_output() {
//!         let context = Test::default();
//!
//!         example(&context).unwrap();
//!
//!         assert_eq!(context.to_output_vec(), b"Hello, world!\n");
//!     }
//! }
//! ```

use std::{io, sync};

/// A generic implementation of [`Context`] that serves as a foundation for more specialized types.
///
/// This generic type implements `Context` so that any type that implements the respective trait
/// for reading ([`io::Read`]) or writing ([`io::Write`]) can be used as an error output, input,
/// or global output stream.
pub struct Base<E, I, O>
where
    E: io::Write + 'static,
    I: io::Read + 'static,
    O: io::Write + 'static,
{
    /// The error output stream.
    error: Shared<E>,

    /// The input stream.
    input: Shared<I>,

    /// The global output stream.
    output: Shared<O>,
}

impl<E, I, O> Context for Base<E, I, O>
where
    E: io::Write,
    I: io::Read,
    O: io::Write,
{
    fn error<C, R>(&self, closure: C) -> R
    where
        C: Fn(&mut dyn io::Write) -> R,
    {
        let mutex = self.error.clone();
        let stream = &mut *mutex
            .lock()
            .expect("Unable to acquire lock for the error output stream.");

        closure(stream)
    }

    fn input<C, R>(&self, closure: C) -> R
    where
        C: Fn(&mut dyn io::Read) -> R,
    {
        let mutex = self.input.clone();
        let stream = &mut *mutex
            .lock()
            .expect("Unable to acquire lock for the input stream.");

        closure(stream)
    }

    fn output<C, R>(&self, closure: C) -> R
    where
        C: Fn(&mut dyn io::Write) -> R,
    {
        let mutex = self.output.clone();
        let stream = &mut *mutex
            .lock()
            .expect("Unable to acquire lock for the global output stream.");

        closure(stream)
    }

    fn to_error(&self) -> Shared<dyn io::Write> {
        self.error.clone()
    }

    fn to_input(&self) -> Shared<dyn io::Read> {
        self.input.clone()
    }

    fn to_output(&self) -> Shared<dyn io::Write> {
        self.output.clone()
    }
}

/// A type implementing this trait provides the input and output streams used by commands.
///
/// As described by the module, there may be cases where using the standard streams may not be
/// possible. To support the use of other streams for error output, input, and global output, the
/// `Context` trait was created.
///
/// ```
/// use carli::error::Result;
/// use carli::io::Context;
///
/// fn example(context: &impl Context) -> Result<()> {
///     context.output(|s| writeln!(s, "Hello, world!"))?;
///
///     Ok(())
/// }
/// ```
pub trait Context {
    /// Locks the handle and passes the closure the error output stream.
    ///
    /// This method will lock the error output handle before handing it over to the closure for
    /// writing. The result of the closure will also be passed through and returned to the caller
    /// of this method. Once the closure exits, the lock on the handle will be release.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Context;
    ///
    /// fn example(context: &impl Context)-> Result<()> {
    ///     context.error(|s| writeln!(s, "Heads up!"))?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn error<C, R>(&self, closure: C) -> R
    where
        C: Fn(&mut dyn io::Write) -> R;

    /// Locks the handle and passes the closure the input stream.
    ///
    /// This method will lock the input handle before handing it over to the closure for reading.
    /// The result of the closure will also be passed through and returned to the caller of this
    /// method. Once the closure exits, the lock on the handle will be release.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Context;
    /// use std::io::{BufReader, Read};
    ///
    /// fn example(context: &impl Context)-> Result<()> {
    ///     let contents: Result<String> = context.input(|s| {
    ///         let mut reader = BufReader::new(s);
    ///         let mut buffer = Vec::new();
    ///
    ///         reader.read_to_end(&mut buffer)?;
    ///
    ///         let contents = String::from_utf8_lossy(buffer.as_slice()).to_string();
    ///
    ///         Ok(contents)
    ///     });
    ///
    ///     if let Ok(string) = contents {
    ///         // Do something with `string`.
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    fn input<C, R>(&self, closure: C) -> R
    where
        C: Fn(&mut dyn io::Read) -> R;

    /// Locks the handle and passes the given the global output stream.
    ///
    /// This method will lock the global output handle before handing it over to the closure for
    /// writing. The result of the closure will also be passed through and returned to the caller
    /// of this method. Once the closure exits, the lock on the handle will be release.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Context;
    ///
    /// fn example(context: &impl Context)-> Result<()> {
    ///     context.output(|s| writeln!(s, "Hello, world!"))?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn output<C, R>(&self, closure: C) -> R
    where
        C: Fn(&mut dyn io::Write) -> R;

    /// Returns a thread-safe mutex for the error output stream.
    ///
    /// There may be cases where more direct access to the error output stream is required, or
    /// concurrent access to both output streams is necessary. For these cases, the mutex guarded
    /// stream can be returned.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Context;
    ///
    /// fn example(context: &impl Context) -> Result<()> {
    ///     let mutex = context.to_error();
    ///     let mut stream = &mut *mutex.lock()?;
    ///
    ///     writeln!(stream, "Heads up!")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn to_error(&self) -> Shared<dyn io::Write>;

    /// Returns a thread-safe mutex for the input stream.
    ///
    /// There may be cases where more direct access to the input stream is required, or concurrent
    /// access to the other output streams is necessary. For these cases, the mutex guarded stream
    /// can be returned.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Context;
    /// use std::io::BufReader;
    ///
    /// fn example(context: &impl Context) -> Result<()> {
    ///     let mutex = context.to_input();
    ///     let stream = &mut *mutex.lock()?;
    ///     let mut buffer = Vec::new();
    ///
    ///     stream.read_to_end(&mut buffer)?;
    ///
    ///     if buffer == b"example" {
    ///         context.output(|s| writeln!(s, "Example matched!"))?;
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    fn to_input(&self) -> Shared<dyn io::Read>;

    /// Returns a thread-safe mutex for the global output stream.
    ///
    /// There may be cases where more direct access to the global output stream is required, or
    /// concurrent access to both output streams is necessary. For these cases, the mutex guarded
    /// stream can be returned.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Context;
    ///
    /// fn example(context: &impl Context) -> Result<()> {
    ///     let mutex = context.to_output();
    ///     let mut stream = &mut *mutex.lock()?;
    ///
    ///     writeln!(stream, "Hello, world!")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn to_output(&self) -> Shared<dyn io::Write>;
}

/// A type for an object behind a mutex that can be shared across threads.
pub type Shared<T> = sync::Arc<sync::Mutex<T>>;

/// A [`Context`] that uses the standard input and output streams.
///
/// ```
/// use carli::outputln;
/// use carli::io::Standard;
///
/// fn main() {
///     let context = Standard::default();
///
///     outputln!(context, "Hello, world!").unwrap();
/// }
/// ```
pub type Standard = Base<io::Stderr, io::Stdin, io::Stdout>;

impl Default for Standard {
    fn default() -> Self {
        Self {
            error: sync::Arc::new(sync::Mutex::new(io::stderr())),
            input: sync::Arc::new(sync::Mutex::new(io::stdin())),
            output: sync::Arc::new(sync::Mutex::new(io::stdout())),
        }
    }
}

/// A [`Context`] that uses in-memory buffers that can be used for testing.
///
/// This `Context` implementation uses in-memory buffers for the error output, input, and global
/// output streams. These buffers can all be accessed for used for reading and writing, through
/// the `Context` APIs or through methods available only through this implementation.
///
/// ```
/// use carli::error::Result;
/// use carli::io::{Context, Test};
/// use carli::outputln;
///
/// fn example(context: &impl Context) -> Result<()> {
///     outputln!(context, "Hello, world!");
///
///     Ok(())
/// }
///
/// #[cfg(test)]
/// mod test {
///     fn example_output() {
///         let context = Test::default();
///
///         example(&context).unwrap();
///
///         assert_eq!(context.to_output_vec(), b"Hello, world!\n");
///     }
/// }
/// ```
pub type Test = Base<Vec<u8>, io::Cursor<Vec<u8>>, Vec<u8>>;

impl Default for Test {
    fn default() -> Self {
        Self {
            error: sync::Arc::new(sync::Mutex::new(Vec::new())),
            input: sync::Arc::new(sync::Mutex::new(io::Cursor::new(Vec::new()))),
            output: sync::Arc::new(sync::Mutex::new(Vec::new())),
        }
    }
}

impl Test {
    /// Resets the in-memory buffer used for the error output, input, and global output streams.
    pub fn reset(&self) {
        self.reset_error();
        self.reset_input();
        self.reset_output();
    }

    /// Resets the in-memory buffer used for the error output stream.
    pub fn reset_error(&self) {
        let mutex = self.error.clone();
        let mut error = mutex.lock().unwrap();

        error.clear();
        error.shrink_to_fit();
    }

    /// Resets the in-memory buffer used for the input stream.
    pub fn reset_input(&self) {
        let mutex = self.input.clone();
        let mut input = mutex.lock().unwrap();

        {
            let inner = input.get_mut();

            inner.clear();
            inner.shrink_to_fit();
        }

        input.set_position(0);
    }

    /// Resets the in-memory buffer used for the global output stream.
    pub fn reset_output(&self) {
        let mutex = self.output.clone();
        let mut output = mutex.lock().unwrap();

        output.clear();
        output.shrink_to_fit();
    }

    /// Sets the contents of the input stream.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::{Context, Test};
    /// use carli::outputln;
    /// use std::io::{BufReader, Read};
    ///
    /// fn example(context: &impl Context) -> Result<()> {
    ///    let input: Result<Vec<u8>> = context.input(|s| {
    ///        let mut buffer = Vec::new();
    ///         let mut reader = BufReader::new(s);
    ///
    ///         reader.read_to_end(&mut buffer)?;
    ///
    ///         Ok(buffer)
    ///     });
    ///
    ///     if let Ok(content) = input {
    ///         let name = String::from_utf8_lossy(content.as_slice());
    ///
    ///         outputln!(context, "Hello, {}!", name);
    ///     }
    ///
    ///     Ok(())
    /// }
    ///
    /// fn main() {
    ///     let context = Test::default();
    ///
    ///     context.set_input(b"example");
    ///
    ///     example(&context).unwrap();
    /// }
    /// ```
    pub fn set_input(&self, contents: &[u8]) {
        let mutex = self.error.clone();
        let mut input = mutex.lock().unwrap();

        *input = Vec::from(contents);
    }

    /// Returns a copy of the in-memory buffer used for the error output stream.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::errorln;
    /// use carli::io::{Context, Test};
    ///
    /// fn example(context: &impl Context) -> Result<()> {
    ///     errorln!(context, "Heads up!")?;
    ///
    ///     Ok(())
    /// }
    ///
    /// #[cfg(test)]
    /// mod test {
    ///     use super::*;
    ///
    ///     fn example_error() {
    ///         let context = Test::default();
    ///
    ///         example(&context).unwrap();
    ///
    ///         assert_eq!(context.to_error_vec(), b"Heads up!\n");
    ///     }
    /// }
    /// ```
    pub fn to_error_vec(&self) -> Vec<u8> {
        let mutex = self.error.clone();
        let vec = &*mutex
            .lock()
            .expect("Unable to acquire lock for the error stream.");

        vec.clone()
    }

    /// Returns a copy of the in-memory buffer used for the global output stream.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::{Context, Test};
    /// use carli::outputln;
    ///
    /// fn example(context: &impl Context) -> Result<()> {
    ///     outputln!(context, "Hello, world!")?;
    ///
    ///     Ok(())
    /// }
    ///
    /// #[cfg(test)]
    /// mod test {
    ///     use super::*;
    ///
    ///     fn example_output() {
    ///         let context = Test::default();
    ///
    ///         example(&context).unwrap();
    ///
    ///         assert_eq!(context.to_output_vec(), b"Hello, world!\n");
    ///     }
    /// }
    /// ```
    pub fn to_output_vec(&self) -> Vec<u8> {
        let mutex = self.output.clone();
        let vec = &*mutex
            .lock()
            .expect("Unable to acquire lock for the output stream.");

        vec.clone()
    }
}

/// Writes to the [`Context`] error output stream.
///
/// A shortcut for writing to the error output stream from a [`Context`].
///
/// ### Using a simple message
///
/// ```
/// use carli::error::Result;
/// use carli::errorln;
/// use carli::io::Context;
///
/// fn example(context: &impl Context) -> Result<()> {
///     errorln!(context, "Heads up!")?;
///
///     Ok(())
/// }
/// ```
///
/// ### Using a formatted message
///
/// ```
/// use carli::error::Result;
/// use carli::errorln;
/// use carli::io::Context;
///
/// fn example(context: &impl Context) -> Result<()> {
///     let name = "world";
///
///     errorln!(context, "Heads up, {}!", name)?;
///
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! errorln {
    ($context:expr, $message:expr) => {{
        use $crate::io::Context;

        $context.error(|s| writeln!(s, $message))
    }};
    ($context:expr, $message:expr, $($args:tt)*) => {{
        use $crate::io::Context;

        $context.error(|s| writeln!(s, $message, $($args)*))
    }};
}

/// Writes to the [`Context`] global output stream.
///
/// A shortcut for writing to the global output stream from a [`Context`].
///
/// ### Using a simple message
///
/// ```
/// use carli::error::Result;
/// use carli::io::Context;
/// use carli::outputln;
///
/// fn example(context: &impl Context) -> Result<()> {
///     outputln!(context, "Hello, world!")?;
///
///     Ok(())
/// }
/// ```
///
/// ### Using a formatted message
///
/// ```
/// use carli::error::Result;
/// use carli::io::Context;
/// use carli::outputln;
///
/// fn example(context: &impl Context) -> Result<()> {
///     let name = "world";
///
///     outputln!(context, "Hello, {}!", name)?;
///
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! outputln {
    ($context:expr, $message:expr) => {{
        use $crate::io::Context;

        $context.output(|s| writeln!(s, $message))
    }};
    ($context:expr, $message:expr, $($args:tt)*) => {{
        use $crate::io::Context;

        $context.output(|s| writeln!(s, $message, $($args)*))
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{BufReader, Read, Write};

    #[test]
    fn base_get_error_mutex() {
        let test = Test::default();

        {
            let mutex = test.to_error();
            let stream = &mut *mutex.lock().unwrap();

            write!(stream, "test").unwrap();
        }

        let mutex = test.error.clone();
        let vec = &*mutex.lock().unwrap();

        assert_eq!(vec, b"test");
    }

    #[test]
    fn base_get_input_mutex() {
        let test = Test::default();

        // Write test content to the vector.
        {
            let mutex = test.input.clone();
            let cursor = &mut *mutex.lock().unwrap();

            write!(cursor, "test").unwrap();

            cursor.set_position(0);
        }

        // Read the contents back out through the API.
        let mutex = test.to_input();
        let stream = &mut *mutex.lock().unwrap();
        let mut buffer = Vec::new();

        stream.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, b"test");
    }

    #[test]
    fn base_get_output_mutex() {
        let test = Test::default();

        {
            let mutex = test.to_output();
            let stream = &mut *mutex.lock().unwrap();

            write!(stream, "test").unwrap();
        }

        let mutex = test.output.clone();
        let vec = &*mutex.lock().unwrap();

        assert_eq!(vec, b"test");
    }

    #[test]
    fn base_read_from_input() {
        let test = Test::default();

        // Write test content to the vector.
        {
            let mutex = test.input.clone();
            let stream = &mut *mutex.lock().unwrap();

            write!(stream, "test").unwrap();

            stream.set_position(0);
        }

        // Read the contents back out through the API.
        let content = test.input(|stream| {
            let mut buffer = Vec::new();
            let mut reader = BufReader::new(stream);

            reader.read_to_end(&mut buffer).unwrap();

            buffer
        });

        assert_eq!(content, b"test");
    }

    #[test]
    fn base_write_to_error() {
        let test = Test::default();

        test.error(|stream| write!(stream, "test")).unwrap();

        let mutex = test.error.clone();
        let stream = &*mutex.lock().unwrap();

        assert_eq!(stream, b"test");
    }

    #[test]
    fn base_write_to_output() {
        let test = Test::default();

        test.output(|stream| write!(stream, "test")).unwrap();

        let mutex = test.output.clone();
        let stream = &*mutex.lock().unwrap();

        assert_eq!(stream, b"test");
    }

    #[test]
    fn test_reset_error() {
        let test = Test::default();

        errorln!(test, "test").unwrap();

        {
            let mutex = test.error.clone();
            let vec = &*mutex.lock().unwrap();

            assert_eq!(vec, b"test\n");
        }

        test.reset_error();

        {
            let mutex = test.error.clone();
            let vec = &*mutex.lock().unwrap();

            assert_eq!(vec, b"");
        }
    }

    #[test]
    fn test_reset_input() {
        let test = Test::default();

        {
            let mutex = test.input.clone();
            let cursor = &mut *mutex.lock().unwrap();

            write!(cursor, "test").unwrap();

            cursor.set_position(0);
        }

        test.input(|s| {
            let mut reader = BufReader::new(s);
            let mut buffer = Vec::new();

            reader.read_to_end(&mut buffer).unwrap();

            assert_eq!(buffer, b"test");
        });

        test.reset_input();

        test.input(|s| {
            let mut reader = BufReader::new(s);
            let mut buffer = Vec::new();

            reader.read_to_end(&mut buffer).unwrap();

            assert_eq!(buffer, b"");
        });
    }

    #[test]
    fn test_reset_output() {
        let test = Test::default();

        outputln!(test, "test").unwrap();

        {
            let mutex = test.output.clone();
            let vec = &*mutex.lock().unwrap();

            assert_eq!(vec, b"test\n");
        }

        test.reset_output();

        {
            let mutex = test.output.clone();
            let vec = &*mutex.lock().unwrap();

            assert_eq!(vec, b"");
        }
    }
}
