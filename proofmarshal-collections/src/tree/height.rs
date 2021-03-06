use std::convert::TryFrom;
use std::num::NonZeroU8;

use thiserror::Error;

use hoard::marshal::{Primitive, blob::*};
use hoard::pointee::{Metadata, MetadataKind};
use proofmarshal_derive::{Commit, Prune};

/// The height of a perfect binary tree.
///
/// Valid range: `0 ..= 63`
#[derive(Commit, Prune, Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Height(u8);

impl Height {
    pub const MAX: u8 = 63;

    #[inline(always)]
    fn assert_valid(&self) {
        assert!(self.0 <= Self::MAX);
    }

    #[inline(always)]
    pub fn new(n: u8) -> Result<Self, TryFromIntError> {
        if n <= Self::MAX {
            Ok(Self(n))
        } else {
            Err(TryFromIntError)
        }
    }

    #[inline(always)]
    pub const unsafe fn new_unchecked(n: u8) -> Self {
        Self(n)
    }

    #[inline(always)]
    pub fn len(self) -> usize {
        self.assert_valid();
        1 << self.0
    }

    #[inline(always)]
    pub fn get(self) -> u8 {
        self.0

    }

    #[inline]
    pub fn try_increment(self) -> Option<NonZeroHeight> {
        if self.0 < Self::MAX {
            Some(NonZeroHeight::new(NonZeroU8::new(self.0 + 1).unwrap()).unwrap())
        } else {
            assert!(self.0 == Self::MAX);
            None
        }
    }
}

hoard::impl_encode_for_primitive!(Height, |this, dst| {
    dst.write_bytes(&[this.0])?
        .finish()
});

hoard::impl_decode_for_primitive!(Height);

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("out of range")]
#[non_exhaustive]
pub struct ValidateHeightError;

impl ValidateBlob for Height {
    type Error = ValidateHeightError;

    #[inline]
    fn validate<'a, V>(blob: BlobCursor<'a, Self, V>) -> Result<ValidBlob<'a, Self>, BlobError<Self::Error, V::Error>>
        where V: PaddingValidator
    {
        blob.validate_bytes(|blob| {
            if blob[0] <= Self::MAX {
                Ok(unsafe { blob.assume_valid() })
            } else {
                Err(ValidateHeightError)
            }
        })
    }
}

impl Primitive for Height {}

hoard::impl_encode_for_primitive!(NonZeroHeight, |this, dst| {
    dst.write_bytes(&[this.0.get()])?
        .finish()
});

hoard::impl_decode_for_primitive!(NonZeroHeight);

impl ValidateBlob for NonZeroHeight {
    type Error = ValidateHeightError;

    #[inline]
    fn validate<'a, V>(blob: BlobCursor<'a, Self, V>) -> Result<ValidBlob<'a, Self>, BlobError<Self::Error, V::Error>>
        where V: PaddingValidator
    {
        blob.validate_bytes(|blob| {
            if 0 < blob[0] && blob[0] <= Height::MAX {
                Ok(unsafe { blob.assume_valid() })
            } else {
                Err(ValidateHeightError)
            }
        })
    }
}

impl Primitive for NonZeroHeight {}

impl Metadata for NonZeroHeight {
    #[inline]
    fn kind(&self) -> MetadataKind {
        MetadataKind::Len(self.0.get() as u64)
    }
}

/// The height of an inner node in a perfect binary tree.
///
/// Valid range: `1 ..= 63`
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonZeroHeight(NonZeroU8);

impl NonZeroHeight {
    #[inline(always)]
    pub fn new(n: NonZeroU8) -> Result<Self, TryFromIntError> {
        if n.get() <= Height::MAX {
            Ok(Self(n))
        } else {
            Err(TryFromIntError)
        }
    }

    #[inline(always)]
    pub const unsafe fn new_unchecked(n: NonZeroU8) -> Self {
        Self(n)
    }

    #[inline]
    pub fn decrement(self) -> Height {
        Height::new(self.0.get().checked_sub(1).unwrap()).unwrap()
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("out of range")]
#[non_exhaustive]
pub struct TryFromIntError;

impl TryFrom<u8> for Height {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(n: u8) -> Result<Self, Self::Error> {
        Self::new(n)
    }
}

impl TryFrom<NonZeroU8> for Height {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(n: NonZeroU8) -> Result<Self, Self::Error> {
        Self::new(n.get())
    }
}

impl TryFrom<usize> for Height {
    type Error = TryFromIntError;

    #[inline]
    fn try_from(n: usize) -> Result<Self, Self::Error> {
        if n <= Height::MAX as usize {
            Ok(Height::new(n as u8).unwrap())
        } else {
            Err(TryFromIntError)
        }
    }
}

impl TryFrom<Height> for NonZeroHeight {
    type Error = TryFromIntError;

    #[inline]
    fn try_from(n: Height) -> Result<Self, Self::Error> {
        NonZeroU8::new(n.0).map(|n| NonZeroHeight(n))
            .ok_or(TryFromIntError)
    }
}

impl TryFrom<usize> for NonZeroHeight {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(n: usize) -> Result<Self, Self::Error> {
        let height = Height::try_from(n)?;
        NonZeroHeight::try_from(height)
    }
}

impl From<Height> for u8 {
    #[inline]
    fn from(height: Height) -> u8 {
        height.0
    }
}

impl From<Height> for usize {
    #[inline]
    fn from(height: Height) -> usize {
        height.0 as usize
    }
}

impl From<NonZeroHeight> for Height {
    #[inline]
    fn from(height: NonZeroHeight) -> Height {
        Self(height.0.get())
    }
}

impl From<NonZeroHeight> for u8 {
    #[inline]
    fn from(height: NonZeroHeight) -> u8 {
        height.0.get()
    }
}

impl From<NonZeroHeight> for usize {
    #[inline]
    fn from(height: NonZeroHeight) -> usize {
        height.0.get() as usize
    }
}

pub unsafe trait GetHeight {
    fn get(&self) -> Height;
}

unsafe impl GetHeight for [()] {
    #[inline]
    fn get(&self) -> Height {
        Height::try_from(self.len()).expect("invalid height")
    }
}

unsafe impl GetHeight for Height {
    #[inline]
    fn get(&self) -> Height {
        *self
    }
}

unsafe impl GetHeight for NonZeroHeight {
    #[inline]
    fn get(&self) -> Height {
        Height::from(*self)
    }
}

unsafe impl GetHeight for () {
    #[inline]
    fn get(&self) -> Height {
        panic!()
    }
}
