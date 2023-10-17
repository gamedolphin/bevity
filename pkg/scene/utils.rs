#[macro_export]
macro_rules! BEVITY_CONST {
    ( $x: ident ) => {
        pub const $x: &str = stringify!($x);
    };
}
