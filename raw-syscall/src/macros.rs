#[macro_export]
macro_rules! cyan {
    ($($arg:tt)*) => ({
        println!("\x1b[36m{}\x1b[0m", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! green {
    ($($arg:tt)*) => ({
        println!("\x1b[32m{}\x1b[0m", format!($($arg)*));
    })
}
