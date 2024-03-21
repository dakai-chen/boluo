/// 将常量字符串声明为类型的宏，该类型实现特征[`Name`]。
///
/// # 例子
///
/// ```
/// use boluo_core::extract::Name;
///
/// boluo_core::name! {
///     #[allow(non_camel_case_types)]
///     pub content_type = "content-type";
/// }
///
/// assert_eq!(content_type::name(), "content-type");
/// ```
///
/// [`Name`]: crate::extract::Name
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
