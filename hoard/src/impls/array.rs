use core::fmt;
use core::mem::{self, MaybeUninit};

use thiserror::Error;

use sliceinit::SliceInitializer;

use super::*;

#[derive(Error, Debug, PartialEq, Eq)]
//#[error("array validation failed at index {idx}: {err}")]
#[error("array validation failed")]
pub struct ValidateArrayError<E: fmt::Debug, const N: usize> {
    idx: usize,
    err: E,
}

impl<E: fmt::Debug + Into<!>, const N: usize> From<ValidateArrayError<E, N>> for ! {
    fn from(err: ValidateArrayError<E,N>) -> ! {
        err.err.into()
    }
}

impl<T: ValidateBlob, const N: usize> ValidateBlob for [T;N] {
    type Error = ValidateArrayError<T::Error, N>;

    fn validate<'a, V: PaddingValidator>(mut blob: BlobCursor<'a, Self, V>)
        -> Result<ValidBlob<'a, Self>, BlobError<Self::Error, V::Error>>
    {
        for i in 0 .. N {
            blob.field::<T,_>(|err| ValidateArrayError { idx: i, err })?;
        }

        unsafe { blob.assume_valid() }
    }
}

unsafe impl<T: Persist, const N: usize> Persist for [T; N] {
    type Persist = [T::Persist; N];
    type Error = <Self::Persist as ValidateBlob>::Error;
}

unsafe impl<'a, Z, T, const N: usize> ValidateChildren<'a, Z> for [T; N]
where T: Persist + ValidateChildren<'a, Z>,
{
    type State = [T::State; N];

    fn validate_children(this: &'a Self::Persist) -> Self::State {
        let mut r: [MaybeUninit<T::State>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut initializer = SliceInitializer::new(&mut r[..]);

        for item in this.iter() {
            initializer.push(T::validate_children(item))
        }

        initializer.done();

        // Need a transmute_copy() as Rust doesn't seem to know the two arrays are the same size.
        let r2 = unsafe { mem::transmute_copy(&r) };
        assert_eq!(mem::size_of_val(&r), mem::size_of_val(&r2));
        assert_eq!(mem::align_of_val(&r), mem::align_of_val(&r2));

        r2
    }

    fn poll<V: PtrValidator<Z>>(this: &'a Self::Persist, state: &mut Self::State, validator: &V) -> Result<(), V::Error> {
        for (item, state) in this.iter().zip(state.iter_mut()) {
            T::poll(item, state, validator)?;
        }
        Ok(())
    }
}

impl<Z, T, const N: usize> Decode<Z> for [T; N]
where T: Decode<Z>,
{}

impl<Y, T: Encoded<Y>, const N: usize> Encoded<Y> for [T; N] {
    type Encoded = [T::Encoded; N];
}

impl<'a, Y, T: Encode<'a, Y>, const N: usize> Encode<'a, Y> for [T; N] {
    type State = [T::State; N];

    fn make_encode_state(&'a self) -> Self::State {
        let mut r: [MaybeUninit<T::State>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut initializer = SliceInitializer::new(&mut r[..]);

        for item in self.iter() {
            initializer.push(item.make_encode_state())
        }

        initializer.done();

        // Need a transmute_copy() as Rust doesn't seem to know the two arrays are the same size.
        let r2 = unsafe { mem::transmute_copy(&r) };
        assert_eq!(mem::size_of_val(&r), mem::size_of_val(&r2));
        assert_eq!(mem::align_of_val(&r), mem::align_of_val(&r2));

        r2
    }

    fn encode_poll<D: Dumper<Y>>(&self, state: &mut Self::State, mut dumper: D) -> Result<D, D::Error> {
        for (item, state) in self.iter().zip(state.iter_mut()) {
            dumper = item.encode_poll(state, dumper)?;
        }
        Ok(dumper)
    }

    fn encode_blob<W: WriteBlob>(&self, state: &Self::State, mut dst: W) -> Result<W::Ok, W::Error> {
        for (item, state) in self.iter().zip(state.iter()) {
            dst = dst.write(item, state)?;
        }
        dst.finish()
    }
}

impl<T: Primitive, const N: usize> Primitive for [T; N] {}

/*
assert_impl_all!([u8;10]: Load<!>);
assert_impl_all!([[bool;10]; 10]: Load<!>);

#[cfg(test)]
mod tests {
    use super::*;

    use core::convert::TryFrom;

    use crate::blob::Bytes;

    #[test]
    fn test() {
        let bytes = Bytes::<[u8;0]>::try_from(&[][..]).unwrap();
        let blob = Blob::from(&bytes).into_cursor();
        Validate::validate(blob).unwrap();

        let bytes = Bytes::<[u8;10]>::try_from(&[0,1,2,3,4,5,6,7,8,9][..]).unwrap();
        let blob = Blob::from(&bytes).into_cursor();
        let valid = Validate::validate(blob).unwrap();
        assert_eq!(valid.to_ref(), &[0,1,2,3,4,5,6,7,8,9]);
    }
}
*/
