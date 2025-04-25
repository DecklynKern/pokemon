#[macro_export]
macro_rules! log {
    ($string:literal $(,$arg:expr)*) => {
        let formatted = format!($string $(,$arg)*);
        println!("{}", formatted);
    }
}