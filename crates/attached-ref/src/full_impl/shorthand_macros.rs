/// An alias for `Result<(AttachableRefFull<..>, _), _>`. Brevity is the sole purpose of this macro.
///
/// # Syntax
/// ```rust,ignore
/// FullResult!('data, 'new_upper, Data, Mapper)
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __FullResult {
    ($data:lifetime, $new_upper:lifetime, $Data:ident, $Mapper:ident) => {
        ::core::result::Result<
            (
                $crate::AttachableRefFull<
                    $data, $new_upper,
                    $Mapper::NewN, $Mapper::NewR, $Mapper::NewM, $Data,
                >,
                $Mapper::Ok,
            ),
            $Mapper::Err,
        >
    };
}

#[doc(inline)]
pub use __FullResult as FullResult;

/// An alias for `Result<(SelfRefSlot<..>, _), _>` where the `RefMut` case is [`Infallible`].
/// Brevity is the sole purpose of this macro.
///
/// # Syntax
/// ```rust,ignore
/// RefResult!('stable, 'new_upper, Mapper)
/// ```
///
/// [`Infallible`]: core::convert::Infallible
#[doc(hidden)]
#[macro_export]
macro_rules! __RefResult {
    ($stable:lifetime, $new_upper:lifetime, $Mapper:ident) => {
        ::core::result::Result<
            (
                $crate::SelfRefSlot<
                    $stable, $new_upper,
                    $Mapper::NewN, $Mapper::NewR, ::core::convert::Infallible,
                >,
                $Mapper::Ok,
            ),
            $Mapper::Err,
        >
    };
}

#[doc(inline)]
pub use __RefResult as RefResult;

/// An alias for `Result<(SelfRefSlot<..>, _), _>` where the `Ref` case is [`Infallible`].
/// Brevity is the sole purpose of this macro.
///
/// # Syntax
/// ```rust,ignore
/// RefMutResult!('stable, 'new_upper, Mapper)
/// ```
///
/// [`Infallible`]: core::convert::Infallible
#[doc(hidden)]
#[macro_export]
macro_rules! __RefMutResult {
    ($stable:lifetime, $new_upper:lifetime, $Mapper:ident) => {
        ::core::result::Result<
            (
                $crate::SelfRefSlot<
                    $stable, $new_upper,
                    $Mapper::NewN, ::core::convert::Infallible, $Mapper::NewM,
                >,
                $Mapper::Ok,
            ),
            $Mapper::Err,
        >
    };
}

#[doc(inline)]
pub use __RefMutResult as RefMutResult;
