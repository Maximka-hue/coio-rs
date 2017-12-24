// Copyright 2015 The coio Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Unix domain socket

use std::io;
use std::os::unix::net::SocketAddr;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::Path;

use mio::Ready;
use mio_uds::UnixListener as MioUnixListener;
use mio_uds::UnixStream as MioUnixStream;

use scheduler::ReadyType;
use super::{make_timeout, GenericEvented, SyncGuard};

macro_rules! create_unix_listener {
    ($inner:expr) => (UnixListener::new($inner, Ready::readable()));
}

macro_rules! create_unix_stream {
    ($inner:expr) => (UnixStream::new($inner, Ready::readable() | Ready::writable()));
}

// macro_rules! create_pipe_reader {
//     ($inner:expr) => (PipeReader::new($inner, Ready::readable()));
// }

// macro_rules! create_pipe_writer {
//     ($inner:expr) => (PipeWriter::new($inner, Ready::writable()));
// }

pub type UnixListener = GenericEvented<MioUnixListener>;

impl UnixListener {
    pub fn bind<P: AsRef<Path>>(path: P) -> io::Result<UnixListener> {
        let inner = try!(MioUnixListener::bind(path.as_ref()));
        create_unix_listener!(inner)
    }

    pub fn accept(&self) -> io::Result<(UnixStream, SocketAddr)> {
        let mut sync_guard = SyncGuard::new();

        loop {
            match self.get_inner().accept() {
                Ok(Some((stream, addr))) => {
                    trace!("UnixListener({:?}): accept() => Ok(..)", self.token);
                    return create_unix_stream!(stream).map(move |s| (s, addr));
                }
                Ok(None) => {
                    trace!("UnixListener({:?}): accept() => WouldBlock", self.token);
                }
                Err(err) => {
                    trace!("UnixListener({:?}): accept() => Err(..)", self.token);
                    return Err(err);
                }
            }

            trace!("UnixListener({:?}): wait(Readable)", self.token);
            sync_guard.disarm();

            match *self.read_timeout.lock() {
                None => self.ready_states.wait(ReadyType::Readable),
                Some(t) => if self.ready_states.wait_timeout(ReadyType::Readable, t) {
                    return Err(make_timeout());
                },
            }
        }
    }

    pub fn try_clone(&self) -> io::Result<UnixListener> {
        let inner = try!(self.get_inner().try_clone());
        create_unix_listener!(inner)
    }
}

impl FromRawFd for UnixListener {
    unsafe fn from_raw_fd(fd: RawFd) -> UnixListener {
        let inner = FromRawFd::from_raw_fd(fd);
        create_unix_listener!(inner).unwrap()
    }
}

pub type UnixStream = GenericEvented<MioUnixStream>;

impl UnixStream {
    pub fn connect<P: AsRef<Path>>(path: &P) -> io::Result<UnixStream> {
        let inner = try!(MioUnixStream::connect(path.as_ref()));
        create_unix_stream!(inner)
    }

    pub fn try_clone(&self) -> io::Result<UnixStream> {
        let inner = try!(self.get_inner().try_clone());
        create_unix_stream!(inner)
    }
}

impl FromRawFd for UnixStream {
    unsafe fn from_raw_fd(fd: RawFd) -> UnixStream {
        let inner = FromRawFd::from_raw_fd(fd);
        create_unix_stream!(inner).unwrap()
    }
}

// pub fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
//     let (reader, writer) = try!(::mio::deprecated::unix::pipe());
//     let reader = try!(create_pipe_reader!(reader));
//     let writer = try!(create_pipe_writer!(writer));
//     Ok((reader, writer))
// }

// pub type PipeReader = GenericEvented<MioPipeReader>;

// impl FromRawFd for PipeReader {
//     unsafe fn from_raw_fd(fd: RawFd) -> PipeReader {
//         let inner = FromRawFd::from_raw_fd(fd);
//         create_pipe_reader!(inner).unwrap()
//     }
// }

// pub type PipeWriter = GenericEvented<MioPipeWriter>;

// impl FromRawFd for PipeWriter {
//     unsafe fn from_raw_fd(fd: RawFd) -> PipeWriter {
//         let inner = FromRawFd::from_raw_fd(fd);
//         create_pipe_writer!(inner).unwrap()
//     }
// }
