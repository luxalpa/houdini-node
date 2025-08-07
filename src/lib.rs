mod attribute_data_basic;
mod attribute_types;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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
pub struct RawAttribute {
    pub tuple_size: usize,
    pub data: RawAttributeData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RawAttributeData {
    Float(Vec<f32>),
    Int(Vec<i32>),
    String(Vec<String>),
    Index(Vec<usize>),
    PrimVertex(Vec<Vec<usize>>),
}

impl RawAttributeData {
    pub fn len(&self) -> usize {
        match self {
            RawAttributeData::Float(v) => v.len(),
            RawAttributeData::Int(v) => v.len(),
            RawAttributeData::String(v) => v.len(),
            RawAttributeData::Index(v) => v.len(),
            RawAttributeData::PrimVertex(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn kind(&self) -> AttributeType {
        match self {
            RawAttributeData::Float(_) => AttributeType::Float,
            RawAttributeData::Int(_) => AttributeType::Int,
            RawAttributeData::String(_) => AttributeType::String,
            RawAttributeData::Index(_) => AttributeType::Index,
            RawAttributeData::PrimVertex(_) => AttributeType::PrimVertex,
        }
    }

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

    pub fn index(self) -> Result<Vec<usize>> {
        match self {
            RawAttributeData::Index(v) => Ok(v),
            other => other.err(AttributeType::Index),
        }
    }

    pub fn prim_vertex(self) -> Result<Vec<Vec<usize>>> {
        match self {
            RawAttributeData::PrimVertex(v) => Ok(v),
            other => other.err(AttributeType::PrimVertex),
        }
    }
}

#[derive(Debug)]
pub enum AttributeType {
    Float,
    Int,
    String,
    Index,
    PrimVertex,
}

impl Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::Float => write!(f, "float"),
            AttributeType::Int => write!(f, "int"),
            AttributeType::String => write!(f, "string"),
            AttributeType::Index => write!(f, "index"),
            AttributeType::PrimVertex => write!(f, "prim_vertex"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No geometry found")]
    NoGeometry,
    #[error("No detail attribute found")]
    NoDetail,
    #[error("Invalid attribute length (expected: {expected}, actual: {actual})")]
    InvalidAttributeLength { expected: usize, actual: usize },
    #[error("Invalid attribute type (expected: {expected}, actual: {actual})")]
    InvalidAttributeType {
        expected: AttributeType,
        actual: AttributeType,
    },
    #[error("Missing geometry at input: {0} ")]
    GeometryMissing(usize),
    #[error("{0}")]
    UserError(String),
    #[error("Input {input_index} missing {entity} attribute: {attr}")]
    MissingAttr {
        input_index: usize,
        entity: EntityKind,
        attr: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn print_json(&self) {
        eprintln!("{}", self.to_string());
    }
}

pub fn load_raw_from_stdin() -> Result<Vec<RawGeometry>> {
    serde_json::from_reader(std::io::stdin()).map_err(Into::into)
}

pub fn generate_to_stdout<G: IntoRawGeometry>(geometry: G) -> Result<()> {
    println!("{}", generate::<G>(geometry)?);
    Ok(())
}

pub fn load_from_raw<G: FromRawGeometry>(
    raw_geometry: RawGeometry,
    input_index: usize,
) -> Result<G> {
    G::from_raw(raw_geometry, input_index)
}

#[cfg(test)]
fn load<G: FromRawGeometry>(reader: impl std::io::Read) -> Result<G> {
    let raw_geometry: Vec<RawGeometry> = serde_json::from_reader(reader)?;
    G::from_raw(raw_geometry.into_iter().next().ok_or(Error::NoGeometry)?, 0)
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
    fn from_raw(raw: RawGeometry, input_index: usize) -> Result<Self>;
}

impl<Pt, Vt, Pr, Dt> FromRawGeometry for Geometry<Pt, Vt, Pr, Dt>
where
    Pt: InAttrs,
    Vt: InAttrs,
    Pr: InAttrs,
    Dt: InAttrs,
{
    fn from_raw(raw: RawGeometry, input_index: usize) -> Result<Self> {
        let mut details = Dt::from_attr(
            raw.detail,
            ErrContext {
                input_index,
                entity: EntityKind::Detail,
            },
        )?;
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
            points: Pt::from_attr(
                raw.points,
                ErrContext {
                    input_index,
                    entity: EntityKind::Point,
                },
            )?
            .collect(),
            vertices: Vt::from_attr(
                raw.vertices,
                ErrContext {
                    input_index,
                    entity: EntityKind::Vertex,
                },
            )?
            .collect(),
            prims: Pr::from_attr(
                raw.prims,
                ErrContext {
                    input_index,
                    entity: EntityKind::Prim,
                },
            )?
            .collect(),
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

#[derive(Debug, Copy, Clone)]
pub struct ErrContext {
    pub input_index: usize,
    pub entity: EntityKind,
}

#[derive(Debug, Copy, Clone)]
pub enum EntityKind {
    Point,
    Vertex,
    Prim,
    Detail,
}

impl Display for EntityKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Point => write!(f, "point"),
            EntityKind::Vertex => write!(f, "vertex"),
            EntityKind::Prim => write!(f, "prim"),
            EntityKind::Detail => write!(f, "detail"),
        }
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

pub trait IntoAttributeDataSource: Sized {
    const LEN: usize;
    fn into_attr_data(data: impl Iterator<Item = Self>) -> RawAttributeData;
}

/// To be derived from the Geo Entity (Point, Vertex, Prim or Detail)
pub trait InAttrs: Sized {
    fn from_attr(
        attrs: HashMap<String, RawAttribute>,
        err_context: ErrContext,
    ) -> Result<impl Iterator<Item = Self>>;

    /// Returns Some(()) for the `()` type, None in all other cases.
    fn empty() -> Option<Self> {
        None
    }
}

impl InAttrs for () {
    fn from_attr(
        _attrs: HashMap<String, RawAttribute>,
        _err_ctx: ErrContext,
    ) -> Result<impl Iterator<Item = Self>> {
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

/// Chunks raw attribute data so that it can be processed more easily by [`FromAttributeData`] into
/// the user-defined types (like glam::Vec3).
pub trait FromAttributeDataSource: Sized {
    const LEN: usize;
    fn from_attr_data(data: RawAttribute) -> Result<impl Iterator<Item = Self>>;
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
