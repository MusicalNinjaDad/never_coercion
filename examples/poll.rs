#![feature(never_type)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{
    io,
    pin::Pin,
    task::{Context, Poll, ready},
};

use bitflags::bitflags;
use futures::Stream;

struct MySocket;

impl PollableSocket for MySocket {
    fn clear_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<!>> {
        todo!()
    }
    
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Ready>> {
        todo!()
    }
}

bitflags! {
    struct Ready: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
    }
}

/// Async polling for a socket
trait PollableSocket
where
    Self: Sized,
{
    /// Clear the readiness state of the underlying socket.
    ///
    /// **This MUST be called after any failed readiness poll.**
    ///
    /// Implementations should attempt to clear the relevant readiness marker of the underlying
    /// socket and then return:
    /// - `Poll::Pending` if successful
    /// - `Poll::Ready(error)` on error, to avoid repeated polling without handling the error
    fn clear_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<!>>;

    /// Check whether the socket is ready.
    ///
    /// ## Note
    ///
    /// You **MUST** call self.clear_ready() in the following cases:
    ///
    /// - If this fails it may leave the socket in an undefined readiness state.
    /// - If you do not make use of the readiness it will remain blocked in that state.
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Ready>>;
}

impl Stream for MySocket {
    type Item = io::Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match ready!(self.as_mut().poll_ready(cx)) {
            Ok(readiness) if readiness.contains(Ready::READ) => todo!("read and stream"),
            _ => self.clear_ready(cx).map_ok(|x| x).map(Some), // <- .map_ok(|x| x) to coerce ! to String
        }
    }
}

fn main() {}
