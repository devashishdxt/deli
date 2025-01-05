use std::{
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use idb::Query;
use serde::Serialize;

use crate::{error::Error, JSON_SERIALIZER};

pub trait Sealed {}

/// Trait for range types.
pub trait RangeType: Sealed {}

/// Denotes a bounded range.
pub struct BoundedRange;

impl RangeType for BoundedRange {}
impl Sealed for BoundedRange {}

/// Denotes an unbounded range.
pub struct UnboundedRange;

impl RangeType for UnboundedRange {}
impl Sealed for UnboundedRange {}

/// Represents a continuous interval over some data type that is used for keys.
pub struct KeyRange<'a, K: ?Sized, R> {
    inner: KeyRangeInner<'a, K, R>,
}

/// Represents a continuous interval over some data type that is used for keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyRangeInner<'a, K: ?Sized, R> {
    Single(&'a K),
    Range(Range<&'a K>),
    RangeInclusive(RangeInclusive<&'a K>),
    RangeFrom(RangeFrom<&'a K>),
    RangeTo(RangeTo<&'a K>),
    RangeToInclusive(RangeToInclusive<&'a K>),
    RangeFull(RangeFull, PhantomData<R>),
}

impl<'a, K: ?Sized, R> From<&'a K> for KeyRange<'a, K, R> {
    fn from(k: &'a K) -> Self {
        Self {
            inner: KeyRangeInner::Single(k),
        }
    }
}

impl<'a, K: ?Sized, R> From<Range<&'a K>> for KeyRange<'a, K, R> {
    fn from(range: Range<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::Range(range),
        }
    }
}

impl<'a, K: ?Sized, R> From<RangeInclusive<&'a K>> for KeyRange<'a, K, R> {
    fn from(range: RangeInclusive<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::RangeInclusive(range),
        }
    }
}

impl<'a, K: ?Sized, R> From<RangeFrom<&'a K>> for KeyRange<'a, K, R> {
    fn from(range: RangeFrom<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::RangeFrom(range),
        }
    }
}

impl<'a, K: ?Sized, R> From<RangeTo<&'a K>> for KeyRange<'a, K, R> {
    fn from(range: RangeTo<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::RangeTo(range),
        }
    }
}

impl<'a, K: ?Sized, R> From<RangeToInclusive<&'a K>> for KeyRange<'a, K, R> {
    fn from(range: RangeToInclusive<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::RangeToInclusive(range),
        }
    }
}

impl<K: ?Sized> From<RangeFull> for KeyRange<'_, K, UnboundedRange> {
    fn from(_: RangeFull) -> Self {
        Self {
            inner: KeyRangeInner::RangeFull(RangeFull, PhantomData),
        }
    }
}

impl<'a, K: ?Sized, R> TryFrom<&KeyRange<'a, K, R>> for Option<Query>
where
    K: Serialize,
    R: RangeType,
{
    type Error = Error;

    fn try_from(value: &KeyRange<'a, K, R>) -> Result<Self, Self::Error> {
        match &value.inner {
            KeyRangeInner::Single(k) => {
                let js_value = k.serialize(&JSON_SERIALIZER)?;
                Ok(Some(Query::Key(js_value)))
            }
            KeyRangeInner::Range(range) => {
                let lower = range.start.serialize(&JSON_SERIALIZER)?;
                let upper = range.end.serialize(&JSON_SERIALIZER)?;

                Ok(Some(Query::KeyRange(idb::KeyRange::bound(
                    &lower,
                    &upper,
                    Some(false),
                    Some(true),
                )?)))
            }
            KeyRangeInner::RangeInclusive(range) => {
                let lower = range.start().serialize(&JSON_SERIALIZER)?;
                let upper = range.end().serialize(&JSON_SERIALIZER)?;

                Ok(Some(Query::KeyRange(idb::KeyRange::bound(
                    &lower,
                    &upper,
                    Some(false),
                    Some(false),
                )?)))
            }
            KeyRangeInner::RangeFrom(range) => {
                let lower = range.start.serialize(&JSON_SERIALIZER)?;

                Ok(Some(Query::KeyRange(idb::KeyRange::lower_bound(
                    &lower,
                    Some(false),
                )?)))
            }
            KeyRangeInner::RangeTo(range) => {
                let upper = range.end.serialize(&JSON_SERIALIZER)?;

                Ok(Some(Query::KeyRange(idb::KeyRange::upper_bound(
                    &upper,
                    Some(true),
                )?)))
            }
            KeyRangeInner::RangeToInclusive(range) => {
                let upper = range.end.serialize(&JSON_SERIALIZER)?;

                Ok(Some(Query::KeyRange(idb::KeyRange::upper_bound(
                    &upper,
                    Some(false),
                )?)))
            }
            KeyRangeInner::RangeFull(_, _) => Ok(None),
        }
    }
}

impl<'a, K: ?Sized> TryFrom<&KeyRange<'a, K, BoundedRange>> for Query
where
    K: Serialize,
{
    type Error = Error;

    fn try_from(value: &KeyRange<'a, K, BoundedRange>) -> Result<Self, Self::Error> {
        Ok(<Option<Query>>::try_from(value)?.unwrap())
    }
}
