# Houdini-node

Write your SOP nodes for [Houdini](https://www.sidefx.com/) in Rust!

This crate is basically a "lite" version of something like the HDK. It does not provide access to any of Houdini's
functions. I wrote this in order to improve my editing and debugging utilities for my game / engine, allowing me to run
some of my game code and immediately get feedback in Houdini when I change some of the input parameters (like
Animation-curve assets).

## Usage

TODO

## Tips and Troubleshooting

### Setting input parameters

Currently, the way to set input parameters is to set them as detail attributes on the input geometry. I suggest wrapping
the node with your own node that sets the detail attributes.

### Increasing the number of input nodes

The Houdini asset currently comes with 5 inputs. If you need more, all you need to do is increase the maximum inputs on
the node asset.

### Glam issues

This crate uses a very generous version range for the `glam` dependency. This only works because this crate uses only
the most basic types and `From` implementations from `glam`. But it's still possible that a larger edit on glam could
break this. If that happens, please open an issue. Also, if you need to use a different crate, such as `bevy_math` or
`nalgebra`, please file an issue and support will be added.

## Future plans and ideas

- Support alternate implementations using Houdini Engine and/or HDK
- File watching from Python.
- Use a different transfer format instead of JSON, such as bgeo or a binary format.
- Setup the nodes parameter interface directly from Rust.
- Possibly also generate the full asset file from a Rust build script.
    - This is partially implemented, but only for the base node.

## Not supported yet

- **Optional attributes in output:**
- **Intrinsics:** This would currently require transferring all of the intrinsics even if they aren't being used.
  This can be worked around by promoting them into actual attributes. In the future, we will probably add a schema for
  the node so that the script can send precisely the data that is expected.
- **Array attributes outside of detail:** We are missing efficient Python functions for this (there's no
  floatListAttribValues
  for all of the values). Could maybe be supported if we use
  bgeo format instead of JSON.
- **Dict attributes** We are most likely going to add these for Detail attributes.