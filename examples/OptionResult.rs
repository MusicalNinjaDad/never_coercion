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

pub fn process_some_try(input: u32) -> Option<io::Result<u32>> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => Some(io_function),
        Err(e) => {
            let o: Option<Result<!, io::Error>> = ignore_blocking(e);

            // It looks like this coerces an Option<io::Result<!>>::None,
            // to an Option<io::Result<u32>>::None
            let r: Result<!, io::Error> = o?;

            Some(try { r? })
        }
    }
}

// Easier to see what is going on if we first explicitly mark the try blocks
// and don't manually construct a final Some()
pub fn process_try_try(input: u32) -> Option<io::Result<u32>> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => Some(io_function),
        Err(e) => {
            try {
                let o: Option<Result<!, io::Error>> = ignore_blocking(e);
                let r: Result<!, io::Error> = o?;
                try { r? }
            }
        }
    }
}

// Which desugars and simplifies to:
pub fn process_desugared(input: u32) -> Option<io::Result<u32>> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => Some(io_function),
        Err(e) => {
            type OptionResultNever = Option<io::Result<!>>;
            type ResultNever = io::Result<!>;
            type OptionResultU32 = Option<io::Result<u32>>;
            type ResultU32 = io::Result<u32>;
            
            let inner_try: ResultU32 = {
                let o: OptionResultNever = ignore_blocking(e);
                #[expect(clippy::question_mark)]
                let r: ResultNever = match o {
                    OptionResultNever::Some(r) => r,
                    // Automatic, hidden, explicit type conversion in desugared version
                    OptionResultNever::None => return OptionResultU32::None,
                };
                match r {
                    // Automatic, hidden, explicit type conversion in desugared version
                    ResultNever::Err(e) => ResultU32::Err(e),
                }
            };

            match inner_try {
                Ok(_) => unreachable!("r came from an io::Result::<!>"),
                // Automatic, hidden, explicit type conversion in desugared version
                Err(_) => OptionResultU32::Some(inner_try),
            }
        }
    }
}

pub fn process_return_result_qmark(input: u32) -> io::Result<u32> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => io_function,
        Err(e) => ignore_blocking(e).unwrap()?,
    }
}

pub fn process_return_result_long(input: u32) -> io::Result<u32> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => io_function,
        Err(e) => {
            let o: Option<Result<!, io::Error>> = ignore_blocking(e);
            let r = o.unwrap();
            let _b = r?;
            #[allow(unreachable_code)]
            _b
        }
    }
}

fn main() {}
