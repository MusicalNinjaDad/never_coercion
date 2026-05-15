# Pre-RFC Never coercion

## Summary

Allow `!` to be used in mainstream code to signify an impossible value without introducing "more work than it's worth".

## Motivation

With the stabilisation of Never (hopefully) just around the corner. We should expect increase use of `!` in the future to explicitly highlight situations which _cannot_ occur. Currently, using `!` to accurately and explicitly anchor this information in the type system and lead to unfortunate foot guns.

In the past 2 months I have run into the following situations where `!` is the _right_ answer, but not the _pragmatic_ answer.

### Async: reset io readiness & Poll::Pending

Before using an io connection it is often necessary to check readiness. These checks can leave the connection in an undesired state and need to be reset if not used.

A related clear function can (semantically) only return `Poll::Pending` or `Poll::Ready(Err)`. Any form of `Poll::Ready(Ok)` is meaningless. As such the _correct_ signature would be `fn clear_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<!>>;`, which fully conveys these semantics without users needing to read the full set of notes in the documentation.

This signature, however, causes issues down the road, for example when implementing `Stream`

```rust
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
            _ => self.clear_ready(cx).map_ok(|x| x).map(Some),
        }
    }
}
```

Note that the call to clear ready needs to be followed by a no-op `.map_ok(|x| x)` in `_ => self.clear_ready(cx).map_ok(|x| x).map(Some)`.

In this case we are lucky that `Poll` offers a convenience function `.map_ok()` to manipulate the wrapped result. Most types do not.

Without this convenience (or the convenience of `ready!`) the code expands to a verbose match:

```rust
_ => match self.clear_ready(cx) {
    Poll::Pending => Poll::Pending,
    Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
}
```

This may seem trivial when reading later. The surrounding code is, by it's very nature, inherently complex; the requirement to add a no-op map adds a completely different dimension of complexity and thus risk, requiring the user to context-switch (I certainly found this cognitively taxing and something that completely threw my focus from the actual implementation).

### Infallible conversions & trait bounds / Option-wrapping
