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
            let r: Result<!, io::Error> = o?; // shorts to a Option<Result<!, io::Error>>::None, which DOES coerce ...
            Some(try { r? })
        }
    }
}

pub fn process_try_try(input: u32) -> Option<io::Result<u32>> {
    let io_function = Ok(input);
    match io_function {
        Ok(_) => Some(io_function),
        Err(e) => {
            try {
                let o: Option<Result<!, io::Error>> = ignore_blocking(e);
                let r: Result<!, io::Error> = o?; // shorts to a Option<Result<!, io::Error>>::None, which DOES coerce ...
                try { r? }
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
