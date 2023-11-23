/// Compile-time string replacement. Can only substitute
/// bytes for bytes (to simplify code).
#[macro_export]
#[doc(hidden)]
macro_rules! __replace {
    ( $input:expr, $from:expr, $to:expr ) => {{
        const OUTPUT_LEN: usize = $input.len();
        const OUTPUT_BUF: [u8; OUTPUT_LEN] =
            $crate::macros::const_exprs::replace($input, $from, $to);

        std::str::from_utf8(&OUTPUT_BUF).unwrap()
    }};
}

pub const fn replace<const N: usize>(input: &str, from: char, to: char) -> [u8; N] {
    let mut buf = clone_bytes::<N>(input.as_bytes());

    let mut i = 0;
    while i < N {
        if buf[i] == from as u8 {
            buf[i] = to as u8;
        }

        i += 1;
    }

    buf
}

const fn clone_bytes<const N: usize>(bytes: &[u8]) -> [u8; N] {
    assert!(bytes.len() == N);

    let mut buf = [0; N];

    let mut i = 0;
    while i < N {
        buf[i] = bytes[i];
        i += 1;
    }

    buf
}
