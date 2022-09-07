use std::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Range, RangeInclusive},
};

use idb::Query;
use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::Serializer;

use crate::{Error, Model};

/// Defines the range of keys
pub struct KeyRange<'a, M, T, K>
where
    M: Model,
    T: Serialize + DeserializeOwned,
    T: Borrow<K>,
    K: Serialize + ?Sized,
{
    inner: KeyRangeInner<'a, K>,
    _generis_model: PhantomData<M>,
    _generics_key_type: PhantomData<T>,
}

enum KeyRangeInner<'a, K>
where
    K: Serialize + ?Sized,
{
    Singe(&'a K),
    Range(Range<&'a K>),
    RangeInclusive(RangeInclusive<&'a K>),
}

impl<'a, M, T, K> From<&'a K> for KeyRange<'a, M, T, K>
where
    M: Model,
    T: Serialize + DeserializeOwned,
    T: Borrow<K>,
    K: Serialize + ?Sized,
{
    fn from(single: &'a K) -> Self {
        Self {
            inner: KeyRangeInner::Singe(single),
            _generis_model: Default::default(),
            _generics_key_type: Default::default(),
        }
    }
}

impl<'a, M, T, K> From<Range<&'a K>> for KeyRange<'a, M, T, K>
where
    M: Model,
    T: Serialize + DeserializeOwned,
    T: Borrow<K>,
    K: Serialize + ?Sized,
{
    fn from(range: Range<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::Range(range),
            _generis_model: Default::default(),
            _generics_key_type: Default::default(),
        }
    }
}

impl<'a, M, T, K> From<RangeInclusive<&'a K>> for KeyRange<'a, M, T, K>
where
    M: Model,
    T: Serialize + DeserializeOwned,
    T: Borrow<K>,
    K: Serialize + ?Sized,
{
    fn from(range: RangeInclusive<&'a K>) -> Self {
        Self {
            inner: KeyRangeInner::RangeInclusive(range),
            _generis_model: Default::default(),
            _generics_key_type: Default::default(),
        }
    }
}

impl<'a, M, T, K> TryFrom<KeyRange<'a, M, T, K>> for Query
where
    M: Model,
    T: Serialize + DeserializeOwned,
    T: Borrow<K>,
    K: Serialize + ?Sized,
{
    type Error = Error;

    fn try_from(key_range: KeyRange<'a, M, T, K>) -> Result<Self, Self::Error> {
        match key_range.inner {
            KeyRangeInner::Singe(singe) => {
                let js_value = singe.serialize(&Serializer::json_compatible())?;
                Ok(Query::Key(js_value))
            }
            KeyRangeInner::Range(range) => {
                let lower = range.start.serialize(&Serializer::json_compatible())?;
                let upper = range.end.serialize(&Serializer::json_compatible())?;

                let key_range = idb::KeyRange::bound(&lower, &upper, Some(false), Some(true))?;

                Ok(Query::KeyRange(key_range))
            }
            KeyRangeInner::RangeInclusive(range) => {
                let lower = range.start().serialize(&Serializer::json_compatible())?;
                let upper = range.end().serialize(&Serializer::json_compatible())?;

                let key_range = idb::KeyRange::bound(&lower, &upper, Some(false), Some(false))?;

                Ok(Query::KeyRange(key_range))
            }
        }
    }
}
