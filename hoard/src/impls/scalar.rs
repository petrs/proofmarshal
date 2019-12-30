use core::num;

use thiserror::Error;

use leint::Le;

use super::*;

macro_rules! impl_all_valid {
    ($($t:ty,)+) => {$(
        impl ValidateBlob for $t {
            type Error = !;
            fn validate<'a, V>(blob: BlobCursor<'a, Self, V>)
                -> Result<ValidBlob<'a, Self>, BlobError<Self::Error, V::Error>>
                where V: PaddingValidator
            {
                unsafe { blob.assume_valid() }
            }
        }

        crate::impl_decode_for_primitive!($t);
    )+}
}

impl_all_valid! {
    !, (),
    u8, Le<u16>, Le<u32>, Le<u64>, Le<u128>,
    i8, Le<i16>, Le<i32>, Le<i64>, Le<i128>,
}

#[non_exhaustive]
#[derive(Error, Debug)]
#[error("invalid bool blob")]
pub struct ValidateBoolError;

impl ValidateBlob for bool {
    type Error = ValidateBoolError;
    fn validate<'a, V>(blob: BlobCursor<'a, Self, V>) -> Result<ValidBlob<'a, Self>, BlobError<Self::Error, V::Error>>
        where V: PaddingValidator
    {
        match blob[0] {
            0 | 1 => unsafe { blob.assume_valid() },
            _ => Err(BlobError::Error(ValidateBoolError)),
        }
    }
}

crate::impl_decode_for_primitive!(bool);

#[non_exhaustive]
#[derive(Debug, Error)]
#[error("non-zero int")]
pub struct ValidateNonZeroIntError;

macro_rules! impl_nonzero {
    ($($t:ty,)+) => {$(
        impl ValidateBlob for $t {
            type Error = ValidateNonZeroIntError;
            fn validate<'a, V>(blob: BlobCursor<'a, Self, V>)
                -> Result<ValidBlob<'a, Self>, BlobError<Self::Error, V::Error>>
                where V: PaddingValidator
            {
                blob.validate_bytes(|blob| {
                    if blob.iter().all(|b| *b == 0) {
                        Err(ValidateNonZeroIntError)
                    } else {
                        Ok(unsafe { blob.assume_valid() })
                    }
                })
            }
        }

        crate::impl_decode_for_primitive!($t);
    )+}
}

impl_nonzero! {
    num::NonZeroU8, Le<num::NonZeroU16>, Le<num::NonZeroU32>, Le<num::NonZeroU64>, Le<num::NonZeroU128>,
    num::NonZeroI8, Le<num::NonZeroI16>, Le<num::NonZeroI32>, Le<num::NonZeroI64>, Le<num::NonZeroI128>,
}