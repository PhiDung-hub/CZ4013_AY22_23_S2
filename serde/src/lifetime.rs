#[macro_export]
macro_rules! extend_lifetime_impl {
    (($($expr:tt)*) as $t:ty) => {{
        let expr = $($expr)*;
        core::mem::transmute::<$t, $t>(expr)
    }};
    (($($expr:tt)*) $next:tt $($rest:tt)*) => {
        extend_lifetime_impl!(($($expr)* $next) $($rest)*)
    };
}

#[macro_export]
macro_rules! extend_lifetime {
    ($($cast:tt)*) => {
        extend_lifetime_impl!(() $($cast)*)
    };
}
