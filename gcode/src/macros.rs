/// Declare some items as requiring the "std" feature flag.
macro_rules! with_std {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "std")]
            #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
            $item
        )*
    }
}
