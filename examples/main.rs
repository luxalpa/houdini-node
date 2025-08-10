use glam::{Mat3, Vec3};
use houdini_node::Geometry;
use houdini_node_macro::{InAttrs, OutAttrs, houdini_node_main};

#[derive(InAttrs, OutAttrs)]
struct MyPoint {
    #[attr(name = "P")]
    position: Vec3,
    // parent: i32,
    // transform: Mat3,
}

#[derive(InAttrs, OutAttrs)]
struct MyVertex {
    ptnum: usize,

    #[attr(name = "N")]
    normal: Vec3,
}

#[derive(InAttrs, OutAttrs)]
struct MyPrim {
    vertices: Vec<usize>,
}

#[derive(InAttrs)]
struct MyDetail {
    my_data: String,
}

#[derive(OutAttrs)]
struct OutDetail {
    some_data: Vec<f32>,
    other_data: String,
}

#[houdini_node_main]
fn my_cool_node(
    geo: Geometry<MyPoint, MyVertex, MyPrim, MyDetail>,
    // _geo2: Geometry<MyPoint>,
) -> Result<Geometry<MyPoint, MyVertex, MyPrim, OutDetail>, String> {
    Ok(Geometry {
        points: geo.points,
        vertices: geo.vertices,
        prims: geo.prims,
        detail: OutDetail {
            some_data: vec![1.0, 2.0, 3.0],
            other_data: "hello".to_string(),
        },
    })
}
