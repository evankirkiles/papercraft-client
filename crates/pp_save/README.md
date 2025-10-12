# pp_save

The `save` crate defines the save file format and provides helpers for working with the file. The actual save file itself is just a `gltf` 2.0 binary (`.glb`) with
our application-specific state encoded in the `extras` root field under
the `papercraft` key.
