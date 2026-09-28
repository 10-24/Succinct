macro_rules! keyspaces {
    ( $( $name:ident, $id:literal, $key:ty => $value:ty ; )+ ) => {
      
        
        #[derive(Debug, Clone, Copy)]
        pub enum KsName {
            $( $name, )+
        }

        impl KsName {
            pub const ALL: &'static [KsName] = &[
                $( KsName::$name, )+
            ];

            pub const fn to_str(&self) -> &'static str {
                match self {
                    $( KsName::$name => $id, )+
                }
            }
        }

        $(
            pub struct $name;
            impl Ks for $name {
                const NAME: &'static str = $id;
                type Key = $key;
                type Value = $value;
            }
        )+
    };
}


pub(crate) use keyspaces;