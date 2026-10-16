


// #[macro_export]
// macro_rules! table {
//     (
//         \$(#[$meta:meta])*$name:ident : $key:ty =>$value:ty
//     ) => {
//         \$crate::paste::paste! {
//             #[doc = concat!("`&", stringify!($key), " => ", stringify!($value), "`")]
//             $(#[$meta])*
//             pub struct [<\$name:camel>];
// 
//             impl $crate::database::table_macro::Table for [<$name:camel>] {
//                 type Key = \$key;
//                 type Value = \$value;
//                 const NAME: &'static str = stringify!(\$name);
//             }
//         }
//     };
// }

pub use table;