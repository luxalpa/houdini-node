use glam::Vec3;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::iter;

/// Re-export itertools as it is used in the derive macros.
pub use itertools;

/// The geometry that gets (de)serialized between Houdini and this script.
#[derive(Debug, Deserialize)]
pub struct RawGeometry {
    pub points: HashMap<String, RawAttribute>,
    pub vertices: HashMap<String, RawAttribute>,
    pub prims: HashMap<String, RawAttribute>,
    pub detail: HashMap<String, RawAttribute>,
}

#[derive(Debug, Serialize)]
pub struct RawGeometryOutput {
    pub points: HashMap<&'static str, RawAttribute>,
    pub vertices: HashMap<&'static str, RawAttribute>,
    pub prims: HashMap<&'static str, RawAttribute>,
    pub detail: HashMap<&'static str, RawAttribute>,
}

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
    pub tuple_size: usize,
    pub data: RawAttributeData,
}

#[derive(Debug)]
pub enum AttributeType {
    Float,
    Int,
    String,
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("No geometry found")]
    NoGeometry,
    #[error("No detail attribute found")]
    NoDetail,
    #[error("Invalid attribute length (expected {expected}, actual {actual})")]
    InvalidAttributeLength { expected: usize, actual: usize },
    #[error("Invalid attribute type (expected {expected}, actual {actual})")]
    InvalidAttributeType {
        expected: AttributeType,
        actual: AttributeType,
    },
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

pub fn load_from_stdin<G: FromRawGeometry>() -> Result<G> {
    load::<G>(std::io::stdin())
}

pub fn load_raw_from_stdin() -> Result<Vec<RawGeometry>> {
    serde_json::from_reader(std::io::stdin()).map_err(Into::into)
}

pub fn generate_to_stdout<G: IntoRawGeometry>(geometry: G) {
    println!("{}", generate::<G>(geometry).unwrap());
}

pub fn load_from_raw<G: FromRawGeometry>(raw_geometry: RawGeometry) -> Result<G> {
    G::from_raw(raw_geometry)
}

fn load<G: FromRawGeometry>(reader: impl std::io::Read) -> Result<G> {
    let raw_geometry: Vec<RawGeometry> = serde_json::from_reader(reader)?;
    G::from_raw(raw_geometry.into_iter().next().ok_or(Error::NoGeometry)?)
}

fn generate<G: IntoRawGeometry>(geometry: G) -> Result<String> {
    let raw_geometry = G::into_raw(geometry)?;
    serde_json::to_string(&raw_geometry).map_err(Into::into)
}

/// The actual geometry for the script to use in AoS (Array-of-structs) form.
#[derive(PartialEq, Debug, Clone)]
pub struct Geometry<Pt, Vt = (), Pr = (), Dt = ()> {
    pub points: Vec<Pt>,
    pub vertices: Vec<Vt>,
    pub prims: Vec<Pr>,
    pub detail: Dt,
}

pub trait FromRawGeometry: Sized {
    fn from_raw(raw: RawGeometry) -> Result<Self>;
}

impl<Pt, Vt, Pr, Dt> FromRawGeometry for Geometry<Pt, Vt, Pr, Dt>
where
    Pt: InAttrs,
    Vt: InAttrs,
    Pr: InAttrs,
    Dt: InAttrs,
{
    fn from_raw(raw: RawGeometry) -> Result<Self> {
        let mut details = Dt::from_attr(raw.detail)?;
        let detail = details.next();

        let detail = if let Some(detail) = detail {
            detail
        } else {
            let Some(v) = Dt::empty() else {
                return Err(Error::NoDetail);
            };
            v
        };

        Ok(Self {
            points: Pt::from_attr(raw.points)?.collect(),
            vertices: Vt::from_attr(raw.vertices)?.collect(),
            prims: Pr::from_attr(raw.prims)?.collect(),
            detail,
        })
    }
}

pub trait IntoRawGeometry: Sized {
    fn into_raw(self) -> Result<RawGeometryOutput>;
}

impl<Pt, Vt, Pr, Dt> IntoRawGeometry for Geometry<Pt, Vt, Pr, Dt>
where
    Pt: OutAttrs,
    Vt: OutAttrs,
    Pr: OutAttrs,
    Dt: OutAttrs,
{
    fn into_raw(self) -> Result<RawGeometryOutput> {
        Ok(RawGeometryOutput {
            points: Pt::into_attr(self.points),
            vertices: Vt::into_attr(self.vertices),
            prims: Pr::into_attr(self.prims),
            detail: Dt::into_attr(vec![self.detail]),
        })
    }
}

pub trait OutAttrs: Sized {
    fn into_attr(entities: Vec<Self>) -> HashMap<&'static str, RawAttribute>;
}

impl OutAttrs for () {
    fn into_attr(_entities: Vec<Self>) -> HashMap<&'static str, RawAttribute> {
        HashMap::new()
    }
}

pub trait IntoAttributeData: Sized {
    type DataType: IntoAttributeDataSource;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType>;
}

impl IntoAttributeData for Vec3 {
    type DataType = [f32; 3];

    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data.map(|v| v.into())
    }
}

impl IntoAttributeData for i32 {
    type DataType = i32;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data
    }
}

impl IntoAttributeData for f32 {
    type DataType = f32;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data
    }
}

impl IntoAttributeData for String {
    type DataType = String;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> impl Iterator<Item = Self::DataType> {
        data
    }
}

pub trait IntoAttributeDataSource: Sized {
    const LEN: usize;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData;
}

impl<const N: usize> IntoAttributeDataSource for [f32; N] {
    const LEN: usize = N;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
        RawAttributeData::Float(data.flatten().collect())
    }
}

impl<const N: usize> IntoAttributeDataSource for [i32; N] {
    const LEN: usize = N;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
        RawAttributeData::Int(data.flatten().collect())
    }
}

impl<const N: usize> IntoAttributeDataSource for [String; N] {
    const LEN: usize = N;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
        RawAttributeData::String(data.flatten().collect())
    }
}

impl IntoAttributeDataSource for f32 {
    const LEN: usize = 1;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
        RawAttributeData::Float(data.collect())
    }
}

impl IntoAttributeDataSource for i32 {
    const LEN: usize = 1;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
        RawAttributeData::Int(data.collect())
    }
}

impl IntoAttributeDataSource for String {
    const LEN: usize = 1;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData {
        RawAttributeData::String(data.collect())
    }
}

/// To be derived from the Geo Entity (Point, Vertex, Prim or Detail)
pub trait InAttrs: Sized {
    fn from_attr(attrs: HashMap<String, RawAttribute>) -> Result<impl Iterator<Item = Self>>;

    /// Returns Some(()) for the `()` type, None in all other cases.
    fn empty() -> Option<Self> {
        None
    }
}

impl InAttrs for () {
    fn from_attr(_attrs: HashMap<String, RawAttribute>) -> Result<impl Iterator<Item = Self>> {
        Ok(iter::empty())
    }

    fn empty() -> Option<Self> {
        Some(())
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

/// Chunks raw attribute data so that it can be processed more easily by [`FromAttributeData`] into
/// the user-defined types (like glam::Vec3).
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
    if attr.tuple_size != T::DataType::LEN {
        return Err(Error::InvalidAttributeLength {
            expected: T::DataType::LEN,
            actual: attr.tuple_size,
        });
    }

    let data_iter = T::DataType::from_attr_data(attr)?;

    Ok(T::from_attr_data(data_iter))
}

pub fn generate_to_attr<T: IntoAttributeData>(data: Vec<T>) -> RawAttribute {
    let data_iter = data.into_iter();
    let data = T::DataType::into_attr_data(T::into_attr_data(data_iter));
    RawAttribute {
        tuple_size: T::DataType::LEN,
        data,
    }
}

#[cfg(test)]
mod tests {
    extern crate self as houdini_node;

    use super::*;
    use glam::Vec3;
    use houdini_node_macro::{InAttrs, OutAttrs};

    #[derive(PartialEq, Debug, Clone, OutAttrs, InAttrs)]
    struct GeoPoint {
        #[attr(name = "P")]
        position: Vec3,
        name: String,
    }

    #[derive(PartialEq, Debug, Clone, OutAttrs, InAttrs)]
    struct GeoDetail {
        some_detail: String,
    }

    #[test]
    fn parsing() {
        let d = r#"
        [
            {
                "points": {
                    "P": {
                        "tuple_size": 3,
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
                        "tuple_size": 1,
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
                "detail": {
                    "some_detail": {
                        "tuple_size": 1,
                        "data": {
                            "string": [
                                "hello"
                            ]
                        }
                    }
                }
            }
        ]
        "#;

        load::<Geometry<GeoPoint>>(d.as_bytes()).unwrap();
    }

    /// Output currently only supports a single geo, but input has multiple.
    fn generate_for_testing<G: IntoRawGeometry>(geometry: G) -> Result<String> {
        let raw_geometry = vec![G::into_raw(geometry)?];
        serde_json::to_string(&raw_geometry).map_err(Into::into)
    }

    #[test]
    fn generating() {
        let g = Geometry::<GeoPoint> {
            points: vec![GeoPoint {
                position: Vec3::ZERO,
                name: "a".to_string(),
            }],
            vertices: vec![],
            prims: vec![],
            detail: (),
        };

        let s = generate_for_testing(g.clone()).unwrap();
        let geo_new = load::<Geometry<GeoPoint>>(s.as_bytes()).unwrap();
        assert_eq!(g, geo_new);
    }
}
