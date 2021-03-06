use core::convert::TryInto;
use core::mem;
use core::num;

use leint::Le;

use super::*;

macro_rules! impl_decode {
    ($t:ty) => {
        unsafe impl<'a, Z> Validate<'a, Z> for $t {
            type State = ();
            fn validate_children(_: &Self) -> () {}
            fn poll<V: PtrValidator<Z>>(this: &'a Self, _: &mut (), _: &V) -> Result<&'a Self, V::Error> {
                Ok(this)
            }
        }
        impl<Z> Decode<Z> for $t {}
    }
}

macro_rules! impl_all_valid {
    ($( $t:ty, )+) => {$(
        impl Persist for $t {
            type Persist = Self;
            type Error = !;

            fn validate_blob<B: BlobValidator<Self>>(blob: B) -> Result<B::Ok, B::Error> {
                blob.validate_bytes(|blob| unsafe { Ok(blob.assume_valid()) })
            }
        }

        impl_decode!($t);
    )+}
}

impl_all_valid! {
    (),
    u8, Le<u16>, Le<u32>, Le<u64>, Le<u128>,
    i8, Le<i16>, Le<i32>, Le<i64>, Le<i128>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct NonZeroIntError;

macro_rules! impl_nonzero {
    ($( $t:ty, )+) => {$(
        impl Persist for $t {
            type Persist = Self;
            type Error = NonZeroIntError;

            fn validate_blob<B: BlobValidator<Self>>(blob: B) -> Result<B::Ok, B::Error> {
                blob.validate_bytes(|blob| {
                    let zeros = [0; mem::size_of::<Self>()];
                    let buf: [u8; mem::size_of::<Self>()] = blob[..].try_into().unwrap();
                    if zeros == buf {
                        Err(NonZeroIntError)
                    } else {
                        unsafe { Ok(blob.assume_valid()) }
                    }
                })
            }
        }

        impl_decode!($t);
    )+}
}

impl_nonzero! {
    num::NonZeroU8, Le<num::NonZeroU16>, Le<num::NonZeroU32>, Le<num::NonZeroU64>, Le<num::NonZeroU128>,
    num::NonZeroI8, Le<num::NonZeroI16>, Le<num::NonZeroI32>, Le<num::NonZeroI64>, Le<num::NonZeroI128>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BoolError(());

impl Persist for bool {
    type Persist = Self;
    type Error = BoolError;

    fn validate_blob<B: BlobValidator<Self>>(blob: B) -> Result<B::Ok, B::Error> {
        unsafe {
            blob.validate_bytes(|blob|
                match &blob[..] {
                    [0] | [1] => Ok(blob.assume_valid()),
                    [_] => Err(BoolError(())),
                    _ => unreachable!(),
                }
            )
        }
    }
}
impl_decode!(bool);

impl Persist for ! {
    type Persist = Self;
    type Error = !;

    fn validate_blob<B: BlobValidator<Self>>(_: B) -> Result<B::Ok, B::Error> {
        panic!()
    }
}
impl_decode!(!);
