#![feature(never_type)]
#![feature(try_blocks)]
#![feature(try_blocks_heterogeneous)]
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
            .map(|e| try { e? }), // currently we need to convert Option<Result<!>> to Option<Result<u32>>
    }
}

// Easier to see what is going on if we explicitly use try blocks
pub fn process_try_try(input: u32) -> Option<io::Result<u32>> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => Some(io_function),
        Err(e) => {
            try {
                let o: Option<io::Result<!>> = ignore_blocking(e);
                
                // It _looks like_ this coerces an Option<io::Result<!>>::None,
                // to an Option<io::Result<u32>>::None, but see below for what
                // is really happening
                let r: io::Result<!> = o?;

                // And this _appears to_ coerce an io::Result<!>::Err to an
                // io::Result<u32>::Err, but, again, see below for reality.
                try { r? }
            }
        }
    }
}

// Which desugars and simplifies to:
pub fn process_desugared(input: u32) -> Option<io::Result<u32>> {
    type OptionResultNever = Option<io::Result<!>>;
    type ResultNever = io::Result<!>;
    type OptionResultU32 = Option<io::Result<u32>>;
    type ResultU32 = io::Result<u32>;

    let io_function = ResultU32::Ok(input);
    match io_function {
        Ok(_) => OptionResultU32::Some(io_function),
        Err(e) => {
            let outer_try: OptionResultU32 = 'outer_try: {
                let o: OptionResultNever = ignore_blocking(e);
                let r: ResultNever = match o {
                    OptionResultNever::Some(r) => r,
                    // Automatic, hidden, explicit type conversion in desugared version
                    OptionResultNever::None => break 'outer_try OptionResultU32::None,
                };
                let inner_try: ResultU32 = match r {
                    // Automatic, hidden, explicit type conversion in desugared version
                    ResultNever::Err(e) => ResultU32::Err(e),
                };
                // Automatic, hidden, explicit type conversion in desugared version
                OptionResultU32::Some(inner_try)
            };

            outer_try
        }
    }
}

// To show the confusion & relevance to ! from a slightly different perspective
pub fn process_return_result_long(input: u32) -> io::Result<u32> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => io_function,
        Err(e) => {
            let o: Option<Result<!, io::Error>> = ignore_blocking(e);
            let r = o.unwrap(); // InferredType `r: io::Result<!>`
            let _b = r?; // InferredType `_b: io::Result<u32>`
            #[allow(unreachable_code)]
            _b // With ! this is unreachable
        }
    }
}

fn ignore_blocking_not_never(err: io::Error) -> io::Result<std::convert::Infallible> {
    Err(err)
}

// Recognition of ! as infallible appears much earlier in process than Infallible
pub fn process_return_result_not_never(input: u32) -> io::Result<u32> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => io_function,
        Err(e) => {
            let r = ignore_blocking_not_never(e); // InferredType `r: io::Result<Infallible>`
            let _b = r?; // InferredType `_b: Infallible`
            Err(io::Error::other("Infallible is not recognised as divergent by HIR, only at MIR"))
        }
    }
}


fn main() {}
