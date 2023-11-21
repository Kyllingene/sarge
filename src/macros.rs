#[macro_export]
macro_rules! argument_result {
    ( $typ:ty ) => { std::result::Result<$typ, <$typ as $crate::ArgumentType>::Error> };
}

#[macro_export]
macro_rules! __parse_arg {
    ( err => $parser:expr, $name:ident ) => {
        let $name = $name.get();
    };

    ( ok => $parser:expr, $name:ident ) => {
        let $name = $name.get().ok();
    };

    ( => $parser:expr, $name:ident ) => {
        let $name = $name.get().unwrap();
    };
}

#[macro_export]
macro_rules! __arg_typ {
    ( err , $typ:ty ) => {
        $crate::argument_result!($typ)
    };

    ( ok , $typ:ty ) => {
        std::option::Option<$typ>
    };

    ( $typ:ty ) => {
        $typ
    };
}

#[macro_export]
macro_rules! __var_tag {
    ( $long:ident ) => { $crate::tag::long(stringify!($long)) };
    ( $short:literal $long:ident ) => { $crate::tag::both($short, stringify!($long)) };
}

#[macro_export]
macro_rules! sarge {
    ( $v:vis $name:ident, $( $( # $spec:ident )? $( $short:literal )? $av:vis $long:ident : $typ:ty ),* $(,)? ) => {
        $v struct $name {
            $( $av $long: $crate::__arg_typ!($($spec,)? $typ), )*
        }
        
        impl $name {
            /// Parse arguments from std::env::args.
            #[allow(unused)]
            pub fn parse() -> Result<(Self, Vec<String>), $crate::ArgParseError> {
                Self::parse_args(std::env::args().collect::<Vec<_>>().as_slice())
            }

            /// Parses the provided arguments.
            #[allow(unused)]
            pub fn parse_args(args: &[String]) -> Result<(Self, Vec<String>), $crate::ArgParseError> {
                let parser = $crate::ArgumentParser::new();

                $(
                    let $long = parser.add::<$typ>($crate::__var_tag!($( $short )? $long));
                )*

                let remainder = parser.parse_args(args)?;

                $(
                    $crate::__parse_arg!($($spec)? => parser, $long);
                )*

                let me = Self {$(
                    $long,
                )*};

                Ok((me, remainder))
            }
        }
    };
}

