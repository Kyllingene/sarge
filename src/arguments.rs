#[macro_export]
macro_rules! error_type {
    ( $typ:ty ) => { <$typ as $crate::ArgumentType>::Error };
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
macro_rules! sarge {
    ( $name:ident, $( $( @ $spec:ident )? $arg:ident : $typ:ty ),+ $(,)? ) => {
        struct $name {
            $( $arg: $typ, )+
        }
        
        impl $name {
            pub fn parse() -> Result<(Self, Vec<String>), $crate::ArgParseError> {
            }

            pub fn parse_provided(
                let parser = $crate::ArgumentParser::new();

                $(
                    let $arg = parser.add($crate::tag::long(stringify!($arg)));
                )+

                let remainder = parser.parse()?;

                $(
                    $crate::__parse_arg!($($spec)? => parser, $arg);
                )+

                let me = Self {$(
                    $arg,
                )+};

                Ok((me, remainder))
            ) {
            }
        }
    };
}

