#[macro_export]
macro_rules! require {
    ($condition: expr, $message: expr) => {
        {
            if !$condition { panic!($message); }
        }
    }
}