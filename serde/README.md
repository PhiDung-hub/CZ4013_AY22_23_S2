# Serialize/Deserialize module
This sub-crate is heavily inspired by [miniserde](https://github.com/dtolnay/miniserde).

Implementation for floating points and unicode is simplified, support for maps are disabled.

`std` features are required by default

## Strategy

1. Define a generic `Place` type, with `Visitor` traits which will be used to iterate through the object and encode/decode

2. Define support type, wrap it around a `Fragment` enum.

3. For JSON support:
  - Serialization: iterate through the object using visitor trait impls on place type. Write to a `Layer` of either primitive types, Sequence, or Map. Concantenate the final strings and handle unicode escapes in the process.
  - Deserialization: iterate through the string, using `[`, `{`, `\` to recognize whether the visitor is inside a sequence or a map. Map it accordingly and reconstruct the struct.

*Note: Since only simple structs are supported, deserialization is guaranteed as long as the deserialized object implements the `Deserialize` trait*
