#[macro_export]
macro_rules! vec {
    // extend syntax
    (..$v:expr, $e:expr) => {{
        let mut temp:Vec<_> = $v;
        temp.push($e);
        temp
    }};
    // Creates a new `Vec` with the given capacity.
    (_;$capacity:expr) => {
        std::vec::Vec::with_capacity($capacity as usize)
    };
    // Fallback: Delegate everything else to the standard library vec!
    ($($json:tt)*) => {
        std::vec![$($json)*]
    };
}
