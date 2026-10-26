#[macro_export]
macro_rules! declare_paths {

 
    (@leaf $name:ident : $alias: literal ;) => {
        pub mod $name {
            pub const NAME: &str = $alias;
            pub const PATH: &str = const_format::concatcp!(super::PATH, self::NAME);
        }
    };

    // Subtree with nested children: e.g., sync_data { ... } optionally followed by ';'
    ($name:ident {$($sub:tt)* }) => {
        pub mod $name {
            const PATH: &str = "";
            crate::declare_paths!(@inner $($sub)*);
        }
    };
    
    ($name:ident : $alias: literal {$($sub:tt)* }) => {
        pub mod $name {
            const PATH: &str = "";
            crate::declare_paths!(@inner $($sub)*);
        }
    };

    (@dir $name:ident : $alias: literal {$($sub:tt)* }) => {
        pub mod $name {
            pub const NAME: &str = $alias;
            pub const PATH: &str = const_format::concatcp!(super::PATH, self::NAME, '/');
            
            crate::declare_paths!(@inner $($sub)*);
        }
    };
    
    // Internal helper: empty base case
    (@inner) => {};

    // Internal helper: consume a leaf item `name;` and recurse on the tail
    (@inner $name:ident ; $($tail:tt)*) => {
        crate::declare_paths!(@leaf $name ;);
        crate::declare_paths!(@inner $($tail)*);
    };

    // Internal helper: consume a leaf item `name;` and recurse on the tail
    (@inner $name:ident : $alias: literal ; $($tail:tt)*) => {
        crate::declare_paths!(@leaf $name: $alias ;);
        crate::declare_paths!(@inner $($tail)*);
    };

    // Internal helper: consume a branch item `name { ... } [;]` and recurse on the tail
    (@inner $name:ident : $alias: literal { $($sub:tt)* }$(;)?) => {
        crate::declare_paths!(@dir $name : $alias { $($sub)* });
    };

    (@inner $name:ident {} $($tail:tt)*) => {
        pub mod $name {
            pub const NAME: &str = stringify!($name);
            pub const PATH: &str = const_format::concatcp!(super::PATH, self::NAME, '/');
        }
        crate::declare_paths!(@inner $($tail)*);
    };

}