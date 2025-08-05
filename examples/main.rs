use glam::Vec3;
use houdini_node::Geometry;
use houdini_node_macro::{InAttrs, OutAttrs, houdini_node};

#[derive(InAttrs, OutAttrs)]
struct MyPoint {
    #[attr(name = "P")]
    position: Vec3,
}

#[houdini_node]
fn my_cool_node(geo: Geometry<MyPoint>, _geo2: Geometry<MyPoint>) -> Result<Geometry<MyPoint>, ()> {
    println!("Hello, world!");
    Ok(geo)
}
