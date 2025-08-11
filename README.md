# Houdini-node

Write your SOP nodes for [Houdini](https://www.sidefx.com/) in Rust!

This crate is basically a "lite" version of something like the HDK. It does not provide access to any of Houdini's
functions. I wrote this in order to improve my editing and debugging utilities for my game / engine, allowing me to run
some of my game code and immediately get feedback in Houdini when I change some of the input parameters (like
Animation-curve assets).

## Plans and ideas

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
- **Dict attributes** We are most likely going to add these.