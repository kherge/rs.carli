/// Provides types used to manage input and output streams used by commands.
///
/// The simplest approach to creating a command is to use the standard input and output streams
/// provided by [`std::io`]. However, using these streams directly becomes an issue when testing
/// is necessary to verify the accuracy in how the streams are used. This module provides types
/// that can be used as a drop-in replacement for those streams while also enabling support for
/// both regular use and testing.
///
/// ### Using the streams individually
///
/// ```no_run
/// use carli::error::Result;
/// use carli::io::Stream;
/// use std::io::{self, Read, Write};
///
/// fn example(mut error: Stream, mut input: Stream, mut output: Stream) -> Result<()> {
///     writeln!(error, "Something went wrong.")?;
///     writeln!(output, "Hello, world!")?;
///
///     let mut buffer = Vec::new();
///
///     input.read_to_end(&mut buffer)?;
///
///     writeln!(output, "{}", String::from_utf8_lossy(&buffer));
///
///     Ok(())
/// }
///
/// fn main() {
///     let error = io::stderr().into();
///     let input = io::stdin().into();
///     let output = io::stdout().into();
///
///     example(error, input, output).unwrap();
/// }
/// ```
///
/// ### Using the streams as a collection
///
/// ```no_run
/// use carli::error::Result;
/// use carli::io::{standard, Streams};
/// use std::io::{self, Read, Write};
///
/// fn example(streams: Streams) -> Result<()> {
///     writeln!(streams.error(), "Something went wrong.")?;
///     writeln!(streams.output(), "Hello, world!")?;
///
///     let mut buffer = Vec::new();
///
///     streams.input().read_to_end(&mut buffer)?;
///
///     writeln!(streams.output(), "{}", String::from_utf8_lossy(&buffer));
///
///     Ok(())
/// }
///
/// fn main() {
///     let streams = standard();
///
///     example(streams).unwrap();
/// }
/// ```
use std::{cell, io};

/// The backing streams that are supported.
#[derive(Debug)]
enum StreamKind {
    /// Uses an in-memory buffer for reading and writing.
    Memory(io::Cursor<Vec<u8>>),

    /// Uses [`io::Stderr`] for writing.
    Stderr(io::Stderr),

    /// Uses [`io::Stdin`] for reading.
    Stdin(io::Stdin),

    /// Uses [`io::Stdout`] for writing.
    Stdout(io::Stdout),
}

/// A stream replacement that supports reading and writing.
///
/// This type is designed to be used in place of the standard streams that come from the standard
/// library: [`io::stderr`], [`io::stdin`], and [`io::stdout`]. The benefit of using this stream
/// becomes apparent when trying to test the contents of the stream during unit testing. If an in-
/// memory buffer is used, the stream can be both read from and written to as needed.
///
/// ### Using the standard streams
///
/// ```no_run
/// use carli::io::Stream;
/// use std::io::{self, Read, Write};
///
/// # fn main() {
/// let mut error: Stream = io::stderr().into();
/// let mut input: Stream = io::stdin().into();
/// let mut output: Stream = io::stdout().into();
///
/// // Write to STDERR.
/// writeln!(error, "Something went wrong.").unwrap();
///
/// // Write to STDOUT.
/// writeln!(output, "Hello, world!").unwrap();
///
/// // Read from STDIN.
/// let content = input.to_string().unwrap();
///
/// println!("{}", content);
/// # }
/// ```
///
/// ### Using an in-memory buffer
///
/// ```
/// use carli::io::Stream;
/// use std::io::{self, Read, Seek, SeekFrom, Write};
///
/// # fn main() {
/// // Start with some data in the buffer.
/// let mut stream: Stream = b"example".to_vec().into();
///
/// // Read from the buffer.
/// let content = stream.to_string().unwrap();
///
/// println!("{}", content);
///
/// // Write to the buffer.
/// stream.seek(SeekFrom::Start(0)).unwrap();
///
/// writeln!(stream, "Hello, world!").unwrap();
///
/// // And read it again.
/// stream.seek(SeekFrom::Start(0)).unwrap();
///
/// let content = stream.to_string().unwrap();
///
/// println!("{}", content);
/// # }
/// ```
#[derive(Debug)]
pub struct Stream {
    /// The backing stream.
    inner: StreamKind,
}

impl From<io::Stderr> for Stream {
    fn from(stderr: io::Stderr) -> Self {
        Self {
            inner: StreamKind::Stderr(stderr),
        }
    }
}

impl From<io::Stdin> for Stream {
    fn from(stdin: io::Stdin) -> Self {
        Self {
            inner: StreamKind::Stdin(stdin),
        }
    }
}

impl From<io::Stdout> for Stream {
    fn from(stdout: io::Stdout) -> Self {
        Self {
            inner: StreamKind::Stdout(stdout),
        }
    }
}

impl From<Vec<u8>> for Stream {
    fn from(buffer: Vec<u8>) -> Self {
        Self {
            inner: StreamKind::Memory(io::Cursor::new(buffer)),
        }
    }
}

impl io::Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            StreamKind::Memory(stream) => stream.read(buf),
            StreamKind::Stdin(stream) => stream.read(buf),
            _ => unimplemented!("The stream does not support reading."),
        }
    }
}

impl io::Seek for Stream {
    fn seek(&mut self, position: io::SeekFrom) -> io::Result<u64> {
        match &mut self.inner {
            StreamKind::Memory(stream) => stream.seek(position),
            _ => unimplemented!("The stream does not support seeking."),
        }
    }
}

impl io::Write for Stream {
    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            StreamKind::Memory(stream) => stream.flush(),
            StreamKind::Stderr(stream) => stream.flush(),
            StreamKind::Stdout(stream) => stream.flush(),
            _ => unimplemented!("The stream does not support flushing."),
        }
    }

    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            StreamKind::Memory(stream) => stream.write(buffer),
            StreamKind::Stderr(stream) => stream.write(buffer),
            StreamKind::Stdout(stream) => stream.write(buffer),
            _ => unimplemented!("The stream does not support writing."),
        }
    }
}

impl Stream {
    /// Reads the stream into a string.
    ///
    /// This method will read from the current position in the stream all the way to the end. The
    /// contents that have been read will then be parsed as a [`String`] and the result is returned
    /// as is.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Stream;
    ///
    /// fn example(stream: &mut Stream) -> Result<()> {
    ///     let string = stream.to_string()?;
    ///
    ///     println!("{}", string);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn to_string(&mut self) -> Result<String, io::Error> {
        String::from_utf8(self.to_buffer()?)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
    }

    /// Reads the stream into a lossy string.
    ///
    /// This method will read from the current position in the stream all the way to the end. The
    /// contents that have been read willl then be parsed as a lossy [`String`] and the result is
    /// returned as is.
    ///
    /// ```
    /// use carli::io::Stream;
    ///
    /// fn example(stream: &mut Stream) {
    ///     let string = stream.to_string_lossy();
    ///
    ///     println!("{}", string);
    /// }
    /// ```
    pub fn to_string_lossy(&mut self) -> String {
        let buffer = self
            .to_buffer()
            .expect("Could not read the stream into the buffer.");

        String::from_utf8_lossy(&buffer).to_string()
    }

    /// Reads the contents of the stream into a buffer.
    ///
    /// This method will read the stream all the way to the end and store the contents in a
    /// buffer that is then returned. If this stream is [`StreamKind::Memory`], the the buffer
    /// position will be reset to the beginning before reading.
    fn to_buffer(&mut self) -> Result<Vec<u8>, io::Error> {
        use std::io::Read;

        let mut buffer = Vec::new();

        self.read_to_end(&mut buffer)?;

        Ok(buffer)
    }
}

/// Manages a collection of input and output streams for a command.
///
/// ```
/// use carli::error::Result;
/// use carli::io::{self, Streams};
/// use std::io::Write;
///
/// fn example(streams: &Streams) -> Result<()> {
///     writeln!(streams.output(), "Hello, world!")?;
///
///     Ok(())
/// }
///
/// fn main() {
///     let streams = io::standard();
///
///     example(&streams).unwrap();
/// }
/// ```
pub struct Streams {
    /// The error output stream.
    error: cell::RefCell<Stream>,

    /// The input stream.
    input: cell::RefCell<Stream>,

    // The global output stream.
    output: cell::RefCell<Stream>,
}

impl Streams {
    /// Returns the error output stream.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Streams;
    /// use std::io::Write;
    ///
    /// fn example(streams: &Streams) -> Result<()> {
    ///     writeln!(streams.error(), "Something is wrong.")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn error(&self) -> cell::RefMut<Stream> {
        self.error.borrow_mut()
    }

    /// Returns the input stream.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Streams;
    ///
    /// fn example(streams: &Streams) -> Result<()> {
    ///     let string = streams.input().to_string()?;
    ///
    ///     println!("{}", string);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn input(&self) -> cell::RefMut<Stream> {
        self.input.borrow_mut()
    }

    /// Returns the global output stream.
    ///
    /// ```
    /// use carli::error::Result;
    /// use carli::io::Streams;
    /// use std::io::Write;
    ///
    /// fn example(streams: &Streams) -> Result<()> {
    ///     writeln!(streams.output(), "Hello, world!")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn output(&self) -> cell::RefMut<Stream> {
        self.output.borrow_mut()
    }

    /// Creates a new instance using the given streams.
    fn new<E, I, O>(error: E, input: I, output: O) -> Self
    where
        E: Into<Stream>,
        I: Into<Stream>,
        O: Into<Stream>,
    {
        Self {
            error: cell::RefCell::new(error.into()),
            input: cell::RefCell::new(input.into()),
            output: cell::RefCell::new(output.into()),
        }
    }
}

/// Creates a new instance of [`Streams`] using in-memory buffers.
///
/// ```
/// use carli::io;
///
/// # fn main() {
/// let streams = io::memory();
/// # }
/// ```
pub fn memory() -> Streams {
    Streams::new(Vec::new(), Vec::new(), Vec::new())
}

/// Creates a new instance of [`Streams`] using the standard streams.
///
/// ```
/// use carli::io;
///
/// # fn main() {
/// let streams = io::standard();
/// # }
/// ```
pub fn standard() -> Streams {
    Streams::new(io::stderr(), io::stdin(), io::stdout())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{Read, Seek, Write};

    fn create_streams() -> Streams {
        Streams {
            error: cell::RefCell::new(Stream {
                inner: StreamKind::Memory(io::Cursor::new(Vec::new())),
            }),
            input: cell::RefCell::new(Stream {
                inner: StreamKind::Memory(io::Cursor::new(Vec::new())),
            }),
            output: cell::RefCell::new(Stream {
                inner: StreamKind::Memory(io::Cursor::new(Vec::new())),
            }),
        }
    }

    #[test]
    fn stream_from_buffer() {
        let _: Stream = Vec::new().into();
    }

    #[test]
    fn stream_from_stderr() {
        let _: Stream = io::stderr().into();
    }

    #[test]
    fn stream_from_stdin() {
        let _: Stream = io::stdin().into();
    }

    #[test]
    fn stream_from_stdout() {
        let _: Stream = io::stdout().into();
    }

    #[test]
    fn stream_read() {
        let mut stream = Stream {
            inner: StreamKind::Memory(io::Cursor::new(b"test".to_vec())),
        };

        let mut buffer = Vec::new();

        stream.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, b"test");
    }

    #[test]
    fn stream_seek() {
        let mut stream = Stream {
            inner: StreamKind::Memory(io::Cursor::new(b"test".to_vec())),
        };

        stream.seek(io::SeekFrom::Start(2)).unwrap();

        let mut buffer = Vec::new();

        stream.read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, b"st");
    }

    #[test]
    fn stream_to_string() {
        let mut stream = Stream {
            inner: StreamKind::Memory(io::Cursor::new(b"test".to_vec())),
        };

        let string = stream.to_string().unwrap();

        assert_eq!(string, "test");
    }

    #[test]
    fn stream_to_string_lossy() {
        let mut stream = Stream {
            inner: StreamKind::Memory(io::Cursor::new(b"test".to_vec())),
        };

        let string = stream.to_string_lossy();

        assert_eq!(string, "test");
    }

    #[test]
    fn stream_write() {
        let mut stream = Stream {
            inner: StreamKind::Memory(io::Cursor::new(Vec::new())),
        };

        write!(stream, "test").unwrap();

        if let StreamKind::Memory(mut cursor) = stream.inner {
            cursor.seek(io::SeekFrom::Start(0)).unwrap();

            let vec = cursor.into_inner();

            assert_eq!(vec, b"test");
        } else {
            assert!(false, "Unexpected StreamKind.");
        }
    }

    #[test]
    fn streams_error() {
        let streams = create_streams();

        write!(streams.error(), "test").unwrap();

        let cursor = match streams.error.into_inner().inner {
            StreamKind::Memory(cursor) => cursor,
            _ => panic!("Expected StreamKind::Memory."),
        };

        assert_eq!(cursor.into_inner(), b"test");
    }

    #[test]
    fn streams_input() {
        let streams = create_streams();

        {
            let mut input = streams.input.borrow_mut();

            write!(input, "test").unwrap();

            input.seek(io::SeekFrom::Start(0)).unwrap();
        }

        let mut buffer = Vec::new();

        streams.input().read_to_end(&mut buffer).unwrap();

        assert_eq!(buffer, b"test");
    }

    #[test]
    fn streams_memory() {
        let _: Streams = memory();
    }

    #[test]
    fn streams_output() {
        let streams = create_streams();

        write!(streams.output(), "test").unwrap();

        let cursor = match streams.output.into_inner().inner {
            StreamKind::Memory(cursor) => cursor,
            _ => panic!("Expected StreamKind::Memory."),
        };

        assert_eq!(cursor.into_inner(), b"test");
    }

    #[test]
    fn streams_standard() {
        let _: Streams = standard();
    }
}
