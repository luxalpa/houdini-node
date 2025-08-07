//! Extra high level attribute types that can be used for fields on the derive macro.

use crate::{FromAttributeData, IntoAttributeData};
use glam::{Mat2, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

// *****************************************

impl FromAttributeData for bool {
    type DataType = i32;
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(|v| v != 0)
    }
}

impl IntoAttributeData for bool {
    type DataType = i32;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(|v| v as i32)
    }
}

// *****************************************

impl FromAttributeData for Vec2 {
    type DataType = [f32; 2];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(Self::from)
    }
}

impl IntoAttributeData for Vec2 {
    type DataType = [f32; 2];
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(Into::into)
    }
}

// *****************************************

impl FromAttributeData for Vec3 {
    type DataType = [f32; 3];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(Self::from)
    }
}

impl IntoAttributeData for Vec3 {
    type DataType = [f32; 3];

    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(Into::into)
    }
}

// *****************************************

impl FromAttributeData for Vec4 {
    type DataType = [f32; 4];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(Self::from)
    }
}

impl IntoAttributeData for Vec4 {
    type DataType = [f32; 4];
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(Into::into)
    }
}

// *****************************************

impl FromAttributeData for Quat {
    type DataType = [f32; 4];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(|[x, y, z, w]| Self::from_xyzw(x, y, z, w))
    }
}

impl IntoAttributeData for Quat {
    type DataType = [f32; 4];
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(|q| [q.x, q.y, q.z, q.w])
    }
}

// *****************************************

impl FromAttributeData for Mat2 {
    type DataType = [f32; 4];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(|arr| Self::from_cols_array(&arr))
    }
}

impl IntoAttributeData for Mat2 {
    type DataType = [f32; 4];
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(|m| m.to_cols_array())
    }
}

// *****************************************

impl FromAttributeData for Mat3 {
    type DataType = [f32; 9];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(|arr| Self::from_cols_array(&arr))
    }
}

impl IntoAttributeData for Mat3 {
    type DataType = [f32; 9];
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(|m| m.to_cols_array())
    }
}

// *****************************************

impl FromAttributeData for Mat4 {
    type DataType = [f32; 16];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(|arr| Self::from_cols_array(&arr))
    }
}

impl IntoAttributeData for Mat4 {
    type DataType = [f32; 16];
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(|m| m.to_cols_array())
    }
}
