use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

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
    pub name: String,
    pub len: usize,
    pub data: RawAttributeData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawGeometry {
    pub points: Vec<RawAttribute>,
    pub vertices: Vec<RawAttribute>,
    pub prims: Vec<RawAttribute>,
    pub detail: Vec<RawAttribute>,
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
        let points = if let Some(point_attr) = d.points.first() {
            let num_points = point_attr.data.len() / point_attr.len;

            let points_map: HashMap<String, RawAttribute> = d
                .points
                .into_iter()
                .map(|attr| (attr.name.clone(), attr))
                .collect();

            (0..num_points)
                .map(|i| Pt::from_attr(&points_map, i))
                .collect::<Result<Vec<_>>>()?
        } else {
            vec![]
        };

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
    fn from_attr(attrs: &HashMap<String, RawAttribute>, index: usize) -> Result<Self>;
}

impl EntityFromAttribute for () {
    fn from_attr(_attrs: &HashMap<String, RawAttribute>, _index: usize) -> Result<Self> {
        Ok(())
    }
}

/// To be implemented by the various data types
pub trait FromAttributeData: Sized {
    const LEN: usize;
    fn from_attr_data(data: &RawAttributeData, index: usize) -> Result<Self>;
}

impl FromAttributeData for Vec3 {
    const LEN: usize = 3;
    fn from_attr_data(data: &RawAttributeData, index: usize) -> Result<Self> {
        match data {
            RawAttributeData::Float(v) => {
                Ok(Vec3::new(v[index * 3], v[index * 3 + 1], v[index * 3 + 2]))
            }
            other => Err(Error::InvalidAttributeType {
                expected: AttributeType::Float,
                actual: other.kind(),
            }),
        }
    }
}

impl FromAttributeData for String {
    const LEN: usize = 1;
    fn from_attr_data(data: &RawAttributeData, index: usize) -> Result<Self> {
        match data {
            RawAttributeData::String(v) => Ok(v[index].clone()),
            other => Err(Error::InvalidAttributeType {
                expected: AttributeType::String,
                actual: other.kind(),
            }),
        }
    }
}

pub fn load_from_attr<T: FromAttributeData>(attr: &RawAttribute, index: usize) -> Result<T> {
    if attr.len != T::LEN {
        return Err(Error::InvalidAttributeLength {
            expected: T::LEN,
            actual: attr.len,
        });
    }

    T::from_attr_data(&attr.data, index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use std::collections::HashMap;

    #[allow(dead_code)] // TODO: Actually check if the data is correct.
    struct GeoPoint {
        position: Vec3,
        name: String,
    }

    impl EntityFromAttribute for GeoPoint {
        fn from_attr(attrs: &HashMap<String, RawAttribute>, index: usize) -> Result<Self> {
            Ok(Self {
                position: load_from_attr(attrs.get("P").unwrap(), index)?,
                name: load_from_attr(attrs.get("name").unwrap(), index)?,
            })
        }
    }

    #[test]
    fn parsing() {
        let d = r#"
        [
            {
                "points": [
                    {
                        "name": "P",
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
                    {
                        "name": "name",
                        "len": 1,
                        "data": {
                            "string": [
                                "a",
                                "b",
                                "c"
                            ]
                        }
                    }
                ],
                "vertices": [],
                "prims": [],
                "detail": []
            }
        ]
        "#;

        load::<Geometry<GeoPoint>>(d.as_bytes()).unwrap();
    }
}
