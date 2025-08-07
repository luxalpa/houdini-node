use crate::{
    FromAttributeData, FromAttributeDataSource, IntoAttributeData, IntoAttributeDataSource,
    RawAttribute, RawAttributeData, Result,
};
use itertools::Itertools;
use std::iter;

macro_rules! impl_attribute_data {
    ($type:ty) => {
        impl FromAttributeData for $type {
            type DataType = $type;
            fn from_attr_data(
                data: impl Iterator<Item = Self::DataType>,
            ) -> impl Iterator<Item = Self> {
                data
            }
        }

        impl IntoAttributeData for $type {
            type DataType = $type;
            fn into_attr_data(
                data: impl Iterator<Item = Self>,
            ) -> impl Iterator<Item = Self::DataType> {
                data
            }
        }
    };
}

macro_rules! impl_attribute_data_source {
    ($type:ty, $variant:ident, $method:ident) => {
        impl FromAttributeDataSource for $type {
            const LEN: usize = 1;
            fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
                Ok(data.data.$method()?.into_iter())
            }
        }

        impl IntoAttributeDataSource for $type {
            const LEN: usize = 1;
            fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
                RawAttributeData::$variant(data.collect())
            }
        }

        impl_attribute_data!($type);
    };
}

impl_attribute_data_source!(f32, Float, float);
impl_attribute_data_source!(i32, Int, int);
impl_attribute_data_source!(String, String, string);
impl_attribute_data_source!(usize, Index, index);
impl_attribute_data_source!(Vec<usize>, PrimVertex, prim_vertex);

macro_rules! impl_array_attribute_data_source {
    ($type:ty, $variant:ident, $method:ident) => {
        impl<const N: usize> FromAttributeDataSource for [$type; N] {
            const LEN: usize = N;
            fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
                Ok(into_array_iter(data.data.$method()?))
            }
        }

        impl<const N: usize> IntoAttributeDataSource for [$type; N] {
            const LEN: usize = N;
            fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
                RawAttributeData::$variant(data.flatten().collect())
            }
        }
    };
}

impl_array_attribute_data_source!(f32, Float, float);
impl_array_attribute_data_source!(i32, Int, int);
impl_array_attribute_data_source!(String, String, string);
impl_array_attribute_data_source!(usize, Index, index);

fn into_array_iter<T, const N: usize>(v: Vec<T>) -> impl Iterator<Item = [T; N]> {
    let mut v = v.into_iter();
    iter::from_fn(move || v.next_array())
}
