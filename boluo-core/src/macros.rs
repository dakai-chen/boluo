#[macro_export]
macro_rules! name {
    () => {};
    ($(#[$attr:meta])* $ident:ident = $name:expr; $($tt:tt)*) => {
        $crate::name! { @imp $(#[$attr])* () $ident = $name; }
        $crate::name! { $($tt)* }
    };
    ($(#[$attr:meta])* pub $ident:ident = $name:expr; $($tt:tt)*) => {
        $crate::name! { @imp $(#[$attr])* (pub) $ident = $name; }
        $crate::name! { $($tt)* }
    };
    ($(#[$attr:meta])* pub ($($vis:tt)*) $ident:ident = $name:expr; $($tt:tt)*) => {
        $crate::name! { @imp $(#[$attr])* (pub ($($vis)*)) $ident = $name; }
        $crate::name! { $($tt)* }
    };
    (@imp $(#[$attr:meta])* ($($vis:tt)*) $ident:ident = $name:expr;) => {
        $(#[$attr])*
        $($vis)* struct $ident;

        impl $crate::extract::Name for $ident {
            fn name() -> &'static str {
                $name
            }
        }
    };
}
