use std::time::SystemTime;

#[macro_export]
macro_rules! debug_drop {
    ( $x:expr $(, $y:expr)* ) => {
        if cfg!(feature = "debug-destructors") {
            eprintln!("\x1B[1;33m[DROP] {}\x1B[0m", format_args!($x, $($y),*));
        }
    }
}

#[macro_export]
macro_rules! debug_create {
    ($x:expr $(, $y:expr)*) => {
        if cfg!(feature = "debug-constructors") {
            eprintln!("\x1B[1;32m[CONS] {}\x1B[0m", format_args!($x, $($y),*));
        }
    }
}

#[macro_export]
macro_rules! debug_define {
    ($x:expr $(, $y:expr)*) => {
        if cfg!(feature = "debug-define") {
            eprintln!("\x1B[1;34m[DEFN] {}\x1B[0m", format_args!($x, $($y),*));
        }
    }
}

#[macro_export]
macro_rules! debug_assign {
    ($x:expr $(, $y:expr)*) => {
        if cfg!(feature = "debug-assign") {
            eprintln!("\x1B[1;35m[ASGN] {}\x1B[0m", format_args!($x, $($y),*));
        }
    }
}

pub fn time<F, T>(id: &str, func: F) -> T
    where F: FnOnce() -> T
{
    if !cfg!(feature = "debug-timings") {
        return func();
    }

    let start = SystemTime::now();
    let out = func();

    let elapsed = start.elapsed().expect("could not get elapsed time");
    let dur = match (elapsed.as_secs(), elapsed.subsec_nanos()) {
        (s, ns) if s > 0 => format!("{}s{}ms", s, ns/1_000_000),
        (_, ns) if ns < 1000 => format!("{}ns", ns),
        (_, ns) if ns < 1_000_000 => format!("{}µs{}ns", ns/1000, ns % 1000),
        (_, ns) => format!("{}ms{}µs", ns/1_000_000, (ns % 1_000_000) / 1000),
    };

    eprintln!("\x1B[1;31m[TIME] {}: {}\x1B[0m", id, dur);

    out
}

