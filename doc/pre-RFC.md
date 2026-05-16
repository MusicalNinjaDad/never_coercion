# Pre-RFC Never coercion

## Summary

Allow `!` to be used in mainstream code to signify an impossible value without introducing "more work than it's worth". Up to now most of my mainstream usage of `!` has brought reduced ergonomics as the cost of accurate typing.

## Motivation

The stabilisation of never is (hopefully) just around the corner (a huuuuge shoutout to [WaffleLapkin](https://github.com/WaffleLapkin) for all its work getting this to the final finish line). We should expect increased use of `!` in the future to explicitly highlight situations which *cannot* occur. Currently, using `!` to accurately and explicitly anchor this information in the type system and lead to unfortunate foot guns.

In the past 2 months I have run into the following situations where `!` is the *right* answer, but not the *pragmatic* answer.

### Examples

#### Async: reset io readiness & Poll::Pending

Before using an io connection it is often necessary to check readiness. These checks can leave the connection in an undesired state and need to be reset if not used.

A related clear function can (semantically) only return `Poll::Pending` or `Poll::Ready(Err)`. Any form of `Poll::Ready(Ok)` is meaningless. As such the *correct_ signature would be `fn clear_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<!>>;`, which fully conveys these semantics without users needing to read the full set of notes in the documentation.

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

#### Infallible conversions & trait bounds

The second case is probably going to be more common in the wild. While implementing a parsing library I:

- Defined a custom error type
- Created a series of custom types to represent the parsed data
- Implemented `FromStr` for those custom types
- Added a basic marker-ish trait `Header`with any type-specific implementation details (e.g. the header key)
- Added `HeaderExt` with a blanket impl to parse the value from a header structure

So far ... nothing magical or unusual. The issue arises around how to handle cases where `FromStr` is infallible.

The *right_ way to do this would be:

```rust
impl FromStr for DeviceType {
    type Err = !;
    ...
```

Then it is clearly defined in the type system that this conversion can never fail, which again fully conveys the semantics without users needing to read the full set of notes in the documentation.

However, this means that the blanket

```rust
impl<H, E> HeaderExt for H
where
    H: Header + FromStr<Err = E>,
    HeaderErr: From<E>,
```

will not trigger.

Here is a full skeleton example

```rust
#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(never_type)]

use std::str::FromStr;

enum HeaderErr {
    ParseError,
}

enum DeviceType {
    AudioController,
    Custom(String),
}

impl FromStr for DeviceType {
    // We have a `Custom` type so this will never fail
    type Err = !;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let devicetype = match s {
            "AudioController" => Self::AudioController,
            _ => Self::Custom(s.to_string()),
        };
        Ok(devicetype)
    }
}

trait Header {}

impl Header for DeviceType {}

trait HeaderExt
where
    Self: Sized,
{
    /// Parse data from a header line ()
    fn parse_header(header: &str) -> Result<Self, HeaderErr>;
}

impl<H, E> HeaderExt for H
where
    H: Header + FromStr<Err = E>,
    HeaderErr: From<E>,
{
    /// Parse data from a header line ()
    fn parse_header(header: &str) -> Result<H, HeaderErr> {
        let (data, checksum) = header.split_once(", sha:").ok_or(HeaderErr::ParseError)?;
        Ok(data.parse()?)
    }
}

fn main() {
    // let device =
    //     DeviceType::parse_header("AudioController, sha:040f4bf53d2ca137d6f767169cdb2fa62849b156");
}

// error[E0599]: the variant or associated item `parse_header` exists for enum `DeviceType`, but its trait bounds were not satisfied
//   --> examples/conversion.rs:57:21
//    |
// 11 | enum DeviceType {
//    | --------------- variant or associated item `parse_header` not found for this enum
// ...
// 57 |         DeviceType::parse_header("AudioController, sha:040f4bf53d2ca137d6f767169cdb2fa62849b156");
//    |                     ^^^^^^^^^^^^ variant or associated item cannot be called on `DeviceType` due to unsatisfied trait bounds
//    |
// note: the following trait bounds were not satisfied:
//       `&DeviceType: Header`
//       `&DeviceType: std::str::FromStr`
//       `&mut DeviceType: Header`
//       `&mut DeviceType: std::str::FromStr`
//       `<&DeviceType as std::str::FromStr>::Err = _`
//       `<&mut DeviceType as std::str::FromStr>::Err = _`
//   --> examples/conversion.rs:45:8
//    |
// 43 | impl<H, E> HeaderExt for H
//    |            ---------     -
// 44 | where
// 45 |     H: Header + FromStr<Err = E>,
//    |        ^^^^^^   ^^^^^^^^^^^^^^^^
//    |        |        |       |
//    |        |        |       unsatisfied trait bound introduced here
//    |        |        unsatisfied trait bound introduced here
//    |        unsatisfied trait bound introduced here
//    = help: items from traits can only be used if the trait is implemented and in scope
// note: `HeaderExt` defines an item `parse_header`, perhaps you need to implement it
//   --> examples/conversion.rs:35:1
//    |
// 35 | trait HeaderExt
//    | ^^^^^^^^^^^^^^^

```

There are two ways around this:

1. (The lazy one) just define

    ```rust
    /// We have a `Custom` type so this will never *actually* fail
    impl FromStr for DeviceType {
        type Err = HeaderErr;
    ...
    ```

1. (The *right* way, which currently compiles but adds another case of future collision with the planned blanket impl in [#64715][TrackingIssue64715]) add

    ```rust
    impl From<!> for HeaderErr {
        fn from(value: !) -> Self {
            match value {}
        }
    }
    ```

#### Option wrapping

It doesn't take a large amount of imagination to envision `Option<Result<!,E>>` or `Option<Result<T,!>>` resulting from similar starting situations to the above examples. Would the recommendation for `Option<Result<!,E>>` be:

- nested maps: `.map(|r| r.map(|never| never))`
- double transposition: `.transpose().map(|_never| None).transpose()`
- map try: `.map(|e| try {e?})`
- Don't use `Result<!,E>` to represent 'only returns on error' but stick with `Result<(),E>` which was used before we had `!`

And for those wondering where this would come from, I originally split out a common error handler in the async example above, but then just inlined it instead:

```rust
#![feature(never_type)]
#![feature(try_blocks)]
use std::io;

fn ignore_blocking(err: io::Error) -> Option<io::Result<!>> {
    match err.kind() {
        // This could just as easily be any error we want to ignore and move on
        // (e.g. `PermissionDenied | ReadOnlyFileSystem | IsADirectory`) when updating
        // "all available files". Possibly with a call to `info!()` to log.
        io::ErrorKind::WouldBlock => None,
        _ => Some(Err(err)),
    }
}

pub fn process(input: u32) -> Option<io::Result<u32>> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => Some(io_function),
        Err(e) => ignore_blocking(e) // hopefully in future we can just add a `,` here
        .map(|e| try {e?}), // currently we need to convert Option<Result<!>> to Option<Result<u32>>
    }
}
```

(Yes the error handler *could* just return `Option<io::Error>` and leave it to the caller to wrap in a `Result`, but wouldn't it be nicer to hand back a type structure that the caller can simply use?)

### Why bother? - there are clear workarounds for each case

`!` is great! It extends the language to provide a clear way to idiomatically express intent. From the point of view of a general language user, I'd consider it as valuable as `None` (is not Null) and `Result` (is neither a tuple nor an exception) in this regard. It therefore deserves a focus on integrative ergonomics in the surrounding language, separately from the core implementation.

1. We should expect `Result`, fallible traits and error-handlers to be the most common cases where people begin to use `!`. If these obvious usages cause "pain" shortly down the road then, sadly, most will simply replace `!` with a dummy value/type and move on.
1. All the reasonable workarounds rely on some form of `map` function. `Poll` makes this easiest by providing a map to the inside `T` of a double-wrapped `Poll<Option<Result<T,E>>>`; `Option` doesn't offer this (for good reasons) but at least has its own `map` which allows chaining. As `Try` nears stabilisation and then gets into stable we should expect an increased number of custom wrapper types; many of which may not think to offer a `map`. This leaves the user stuck with verbose match destructuring; or avoiding either `!` or the custom try type (or both).

### Ergonomics

The 2017 [[Ergonomics Initiative]] lays out 3 dimensions to balance when looking at providing implicitness for reasons of ergonomics.

#### Applicability (4/5)

- Strictly excluding `match` etc. from consideration removes the side-effects that made previous considerations impossible at the cost of slightly reduced applicability.
- The coercion is restricted to only cover situations with `<!>` as a generic type, generic type bound or an associated type.

#### Power (2/5)

- Converting *from* `Foo<!>` to `Foo<T>` will never destroy any information. Or rather, the implicit conversion will only take effect *if it is safe to do so*.
- By performing this as part of the type-safety & generics analysis no runtime conversion of data occurs.
- No memory access or implicit dereferencing occurs.

#### Context-Dependence (2/5)

- By restricting to situations where type-inference is already expected the overall influence is restricted to *at most* the current function / trait impl boundary as return types are always explicit. The user only needs to look at two function / trait signatures which are immediately adjacent to the current code to see `!` incoming and `T` outgoing.
- Additionally rust-analyzer is commonly used and provides inline details of the explicit & inferred types directly in place in the code for most users.

[TrackingIssue64715]: (https://github.com/rust-lang/rust/issues/64715)
[Ergonomics Initiative]: (https://blog.rust-lang.org/2017/03/02/lang-ergonomics/#how-to-analyze-and-manage-the-reasoning-footprint)
