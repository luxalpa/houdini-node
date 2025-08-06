use glam::Vec3;
use houdini_node::Geometry;
use houdini_node_macro::{InAttrs, OutAttrs, houdini_node_main};

#[derive(InAttrs, OutAttrs)]
struct MyPoint {
    #[attr(name = "P")]
    position: Vec3,
}

#[houdini_node_main]
fn my_cool_node(
    geo: Geometry<MyPoint>,
    _geo2: Geometry<MyPoint>,
) -> Result<Geometry<MyPoint>, String> {
    println!("Hello, world!");
    Ok(geo)
}
