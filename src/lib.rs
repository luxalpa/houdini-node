use glam::Vec3;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::iter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawAttributeData {
    Float(Vec<f32>),
    Int(Vec<i32>),
    String(Vec<String>),
}

impl RawAttributeData {
    pub fn len(&self) -> usize {
        match self {
            RawAttributeData::Float(v) => v.len(),
            RawAttributeData::Int(v) => v.len(),
            RawAttributeData::String(v) => v.len(),
        }
    }

    pub fn kind(&self) -> AttributeType {
        match self {
            RawAttributeData::Float(_) => AttributeType::Float,
            RawAttributeData::Int(_) => AttributeType::Int,
            RawAttributeData::String(_) => AttributeType::String,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawAttribute {
    pub len: usize,
    pub data: RawAttributeData,
}

impl RawAttributeData {
    /// Helper function
    fn err<T>(self, expected: AttributeType) -> Result<T> {
        Err(Error::InvalidAttributeType {
            expected,
            actual: self.kind(),
        })
    }

    pub fn float(self) -> Result<Vec<f32>> {
        match self {
            RawAttributeData::Float(v) => Ok(v),
            other => other.err(AttributeType::Float),
        }
    }

    pub fn int(self) -> Result<Vec<i32>> {
        match self {
            RawAttributeData::Int(v) => Ok(v),
            other => other.err(AttributeType::Int),
        }
    }

    pub fn string(self) -> Result<Vec<String>> {
        match self {
            RawAttributeData::String(v) => Ok(v),
            other => other.err(AttributeType::String),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawGeometry {
    pub points: HashMap<String, RawAttribute>,
    pub vertices: HashMap<String, RawAttribute>,
    pub prims: HashMap<String, RawAttribute>,
    pub detail: HashMap<String, RawAttribute>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("No geometry found")]
    NoGeometry,
    #[error("Invalid attribute length (expected {expected}, actual {actual})")]
    InvalidAttributeLength { expected: usize, actual: usize },
    #[error("Invalid attribute type (expected {expected}, actual {actual})")]
    InvalidAttributeType {
        expected: AttributeType,
        actual: AttributeType,
    },
}

#[derive(Debug)]
pub enum AttributeType {
    Float,
    Int,
    String,
}

impl Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::Float => write!(f, "float"),
            AttributeType::Int => write!(f, "int"),
            AttributeType::String => write!(f, "string"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn load_from_stdin<G: GeometryTrait>() {
    load::<G>(std::io::stdin()).unwrap();
}

fn load<G: GeometryTrait>(reader: impl std::io::Read) -> Result<G> {
    let raw_geometry: Vec<RawGeometry> = serde_json::from_reader(reader)?;
    G::from_raw(raw_geometry)
}

pub trait GeometryTrait: Sized {
    fn from_raw(raw: Vec<RawGeometry>) -> Result<Self>;
}

pub struct Geometry<Pt, Vt = (), Pr = (), Dt = ()> {
    pub points: Vec<Pt>,
    pub vertices: Vec<Vt>,
    pub prims: Vec<Pr>,
    pub detail: Vec<Dt>,
}

impl<Pt, Vt, Pr, Dt> GeometryTrait for Geometry<Pt, Vt, Pr, Dt>
where
    Pt: EntityFromAttribute,
    Vt: EntityFromAttribute,
    Pr: EntityFromAttribute,
    Dt: EntityFromAttribute,
{
    fn from_raw(raw: Vec<RawGeometry>) -> Result<Self> {
        let d = raw.into_iter().next().ok_or(Error::NoGeometry)?;
        let points = Pt::from_attr(d.points)?;

        Ok(Self {
            points,
            vertices: Vec::new(),
            prims: Vec::new(),
            detail: Vec::new(),
        })
    }
}

/// To be derived from the Geo Entity (Point, Vertex, Prim or Detail)
pub trait EntityFromAttribute: Sized {
    fn from_attr(attrs: HashMap<String, RawAttribute>) -> Result<Vec<Self>>;
}

impl EntityFromAttribute for () {
    fn from_attr(_attrs: HashMap<String, RawAttribute>) -> Result<Vec<Self>> {
        Ok(vec![])
    }
}

/// Translates from the chunked raw data into the final representation.
/// To be implemented by the various data types.
pub trait FromAttributeData: Sized {
    type DataType: FromAttributeDataSource;
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self>;
}

impl FromAttributeData for Vec3 {
    type DataType = [f32; 3];
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data.map(Self::from)
    }
}

impl FromAttributeData for i32 {
    type DataType = i32;
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data
    }
}

impl FromAttributeData for f32 {
    type DataType = f32;
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data
    }
}

impl FromAttributeData for String {
    type DataType = String;
    fn from_attr_data(data: impl Iterator<Item = Self::DataType>) -> impl Iterator<Item = Self> {
        data
    }
}

pub trait FromAttributeDataSource: Sized {
    const LEN: usize;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>>;
}

fn into_array_iter<T, const N: usize>(v: Vec<T>) -> impl Iterator<Item = [T; N]> {
    let mut v = v.into_iter();
    iter::from_fn(move || {
        if let Some(val) = v.next_array() {
            Some(val)
        } else {
            None
        }
    })
}

impl<const N: usize> FromAttributeDataSource for [f32; N] {
    const LEN: usize = N;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
        Ok(into_array_iter(data.data.float()?))
    }
}

impl<const N: usize> FromAttributeDataSource for [i32; N] {
    const LEN: usize = N;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
        Ok(into_array_iter(data.data.int()?))
    }
}

impl<const N: usize> FromAttributeDataSource for [String; N] {
    const LEN: usize = N;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
        Ok(into_array_iter(data.data.string()?))
    }
}

impl FromAttributeDataSource for f32 {
    const LEN: usize = 1;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
        Ok(data.data.float()?.into_iter())
    }
}

impl FromAttributeDataSource for i32 {
    const LEN: usize = 1;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
        Ok(data.data.int()?.into_iter())
    }
}

impl FromAttributeDataSource for String {
    const LEN: usize = 1;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>> {
        Ok(data.data.string()?.into_iter())
    }
}

pub fn load_from_attr<T: FromAttributeData>(attr: RawAttribute) -> Result<impl Iterator<Item = T>> {
    if attr.len != T::DataType::LEN {
        return Err(Error::InvalidAttributeLength {
            expected: T::DataType::LEN,
            actual: attr.len,
        });
    }

    let data_iter = T::DataType::from_attr_data(attr)?;

    Ok(T::from_attr_data(data_iter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use itertools::izip;
    use std::collections::HashMap;

    #[allow(dead_code)] // TODO: Actually check if the data is correct.
    struct GeoPoint {
        position: Vec3,
        name: String,
    }

    impl EntityFromAttribute for GeoPoint {
        fn from_attr(mut attrs: HashMap<String, RawAttribute>) -> Result<Vec<Self>> {
            let positions = load_from_attr(attrs.remove("P").unwrap())?;
            let names = load_from_attr(attrs.remove("name").unwrap())?;

            Ok(izip!(positions, names)
                .map(|(position, name)| Self { position, name })
                .collect())
        }
    }

    #[test]
    fn parsing() {
        let d = r#"
        [
            {
                "points": {
                    "P": {
                        "len": 3,
                        "data": {
                            "float": [
                                0.0,
                                0.0,
                                0.0,
                                1.0,
                                0.0,
                                0.0,
                                1.0,
                                1.0,
                                0.0
                            ]
                        }
                    },
                    "name": {
                        "len": 1,
                        "data": {
                            "string": [
                                "a",
                                "b",
                                "c"
                            ]
                        }
                    }
                },
                "vertices": {},
                "prims": {},
                "detail": {}
            }
        ]
        "#;

        load::<Geometry<GeoPoint>>(d.as_bytes()).unwrap();
    }
}
