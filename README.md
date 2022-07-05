# TSON

A parser for "TSON", a JSON-like object data-interchange format without the usage of `null`.

## TSON examples

```json
{
    "error_code": Some(400),
    "body": None
}
```

```json
{
    "error_code": None,
    "body": Some({
         "response": "bleblebleble",
    })
}
```

## Usage

Currently the only entrypoint is the `parse_value`, which receives a string slice and returns a `Value`, which is defined as shown below.

```rust
pub enum Value<'a> {
    Float(f64),
    Boolean(bool),
    String(&'a str),
    Char(char),
    List(Vec<Value<'a>>),
    Optional(Option<Box<Value<'a>>>),
    Object(HashMap<&'a str, Value<'a>>),
}
```
