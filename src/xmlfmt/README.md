# Conversion specification between XML-RPC and Serde

Serde's [documentation](https://serde.rs/data-model.html) states the data formats it uses to convert data to Rust types. XML-RPC has a very limited set of types, so there will be a lot of aliasing. Here are the connections:

* bool - boolean
* `i8`, `i16`, `i32`, `u8`, `u16` - `int`
* `u64`, `u32`, `u64` - `string`
* `f32`, `f64` - `double`
* `char`, `string` - `string`
* `byte array [u8]` - `base64`
* `option` - array that is either empty or has this one element
* `unit`, `unit_struct` - empty `struct`
* `newtype_struct` - treat as just its content
* `newtype_variant`, `unit_variant`, `tuple_variant`, `struct_variant` - `struct` with one element, whose name is the name of the variant. The content corresponds to the fitting real content
* `seq`, `tuple`, `tuple_struct` - `array`
* `map`, `struct` - `struct`
