#![expect(unsafe_code, reason = "const-hack. TODO: Someday, this `unsafe` can be removed")]

#![expect(clippy::undocumented_unsafe_blocks, reason = "TODO")]

use core::hint::unreachable_unchecked;
use core::mem::{ManuallyDrop, MaybeUninit};

use crate::SelfRefCases;


/// # Robust Guarantees
/// The body of this function is equivalent to a `Deref` coercion of `value`.
///
/// It is merely done in a const-compatible way.
///
/// This function does not unwind (unless UB has already occurred).
#[inline]
#[must_use]
pub(crate) const fn deref_const_hack<T>(value: &ManuallyDrop<T>) -> &T {
    let value_raw: *const ManuallyDrop<T> = value;
    // Since `ManuallyDrop` is stably guaranteed to be `repr(transparent)`, we know that
    // `value_inner` is a pointer (with valid read provenance) to `*value`, which is the
    // value of type `T` wrapped in the `*value` value of `ManuallyDrop`.
    let value_inner: *const T = value_raw.cast();

    // SAFETY:
    // - `value_inner` is properly-aligned and non-null, since its address is equal to `value`,
    //   which is properly aligned and non-null.
    // - `value_inner` is dereferenceable, because it has provenance over the same allocation
    //   as `value` (and the range of memory it points to is the same range of memory as `value`),
    //   and `value` is dereferenceable.
    // - The pointee of `value_inner` is valid for type `T`, since the pointee of `value` must be
    //   valid for type `ManuallyDrop<T>`, implying that the `T` value that it transparently wraps
    //   must be valid.
    // - The aliasing rules are satisfied, since this is equivalent to a `Deref` coercion. More
    //   formally, the aliasing guarantees of `value` for lifetime `'_` imply the necessary
    //   aliasing guarantees for `&*value_inner` for lifetime `'_`.
    unsafe { &*value_inner }
}

/// # Robust Guarantees
/// This function does not unwind (unless UB has already occurred).
///
/// When this function returns, `tuple.0` has been written to `left_out`, and `tuple.1` has been
/// written to `right_out`.
///
/// In other words, both outpointers are always initialized by this function.
#[inline]
pub(crate) const fn split_tuple_const_hack<T, U>(
    tuple:     (T, U),
    left_out:  &mut MaybeUninit<T>,
    right_out: &mut MaybeUninit<U>,
) {
    let tuple = ManuallyDrop::new(tuple);
    let tuple_ref = deref_const_hack(&tuple);

    let left_src = &raw const tuple_ref.0;
    let left_dst = left_out.as_mut_ptr();

    unsafe {
        left_src.copy_to_nonoverlapping(left_dst, 1);
    };

    let right_src = &raw const tuple_ref.1;
    let right_dst = right_out.as_mut_ptr();

    unsafe {
        right_src.copy_to_nonoverlapping(right_dst, 1);
    }
}

macro_rules! split_tuple_const_hack_macro {
    ($tuple:ident, $left:ident, $right:ident) => {
        let mut left = ::core::mem::MaybeUninit::uninit();
        let mut right = ::core::mem::MaybeUninit::uninit();

        $crate::const_hack::split_tuple_const_hack($tuple, &mut left, &mut right);

        let $left = unsafe { left.assume_init() };
        let $right = unsafe { right.assume_init() };
    };
}

pub(crate) use split_tuple_const_hack_macro as split_tuple_const_hack;

/// # Safety
/// `matches!(slot, SelfRefCases::NoRef(_))` must be `true`.
pub(crate) const unsafe fn unwrap_no_ref_unchecked_const_hack<N, R, M>(
    slot: SelfRefCases<N, R, M>,
) -> N {
    let slot_ref = &slot;

    let no_ref_byte_offset = match slot_ref {
        SelfRefCases::NoRef(no_ref)     => {
            let base: *const SelfRefCases<N, R, M> = slot_ref;
            let base: *const u8 = base.cast();

            let field: *const N = no_ref;
            let field: *const u8 = field.cast();

            unsafe { field.offset_from(base) }
        }
        SelfRefCases::Ref(_) | SelfRefCases::RefMut(_) => unsafe { unreachable_unchecked() },
    };

    let slot = ManuallyDrop::new(slot);

    let wrapper_raw: *const ManuallyDrop<SelfRefCases<N, R, M>> = &raw const slot;
    let offset = unsafe { wrapper_raw.byte_offset(no_ref_byte_offset) };

    let no_ref_raw: *const N = offset.cast();

    unsafe { no_ref_raw.read() }
}
