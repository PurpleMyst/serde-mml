# MML spec

MML (Markdown Markup Language) is a revolutionary new markup language which allows arbitrary serialization of data, like all your favorite markup languages such as JSON or JSON5, and deserialization.

Its advantages include but are **definitely** not limited to:

- UTF-8 text! You can send this anywhere that expects text
- Human readable! open it and inspect your data like the good old days!
- Lightweight! Serialized documents are just mere megabytes!

You can find an example serialization comparison in the files `SP.POP.TOTL.json` (taken from https://github.com/jdorfman/awesome-json-datasets) and `SP.POP.TOTL.md`.
You may notice that the MML representation is 12.8 times bigger.. that's fine! It means you can put that 2TB hard-drive to use ;)

The default binary target allows you to transcode between JSON and MML via stdin/stdout respectively.

## Type URIs

Values are given typing utilizing a special form of URI

Here's what the different parts mean:

1. `serde://` => Scheme name, nothing to see here.
2. `DOMAIN` => Represents the "archetype" in the serde data model (e.g. `struct`, `unit_variant`, ...)
3. `/PATH` => Different types implement this differently, but for example sequences encode the length (if known) here

## Serde Data Model

The following section describes how all of the Serde data model is serialized, mostly by example.

Oftentimes the examples are not 100% correct for brevity's sake, e.g. sometimes I've just written a number out

### bool

Serialized as `true` or `false`

    [true](serde://bool)
    [false](serde://bool)

### u8, u16, u32, u64, i8, i16, i32, i64, f32, f64

Serialized as their textual form

    [8](serde://u64)

### char

Serialized as their character value, escaped if necessary

    [c](serde://char)

### String

Serialized as UTF-8 text, escaped if necessary (e.g. \*wow\*);  
NUL bytes should not be a problem

    [foo bar](serde://string)
    [baz \*wow\*](serde://string)

### [u8]

Serialized as a url-safe base64 string

    [d2hhdCBkaWQgeW91IGp1c3Qgc2F5IGFib3V0IG1lPw==](serde://blob)

### unit

Serialized as a special value, like bool

    [()](serde://unit)

### Option

If the option is `None`, it is serialized as a singleton value:

    [none](serde://option/none)

If the option matches `Some(x)`, it is serialized as a newtype struct variant:

    0. [Some](serde://option/some)
    1. x

### Unit Struct

Serialized as its name

    [Unit](serde://unit_struct/Unit)

### Unit Variant

Serialized as the the variant name

Type contains enum name

    [A](serde://unit_variant/E/A)

### Newtype Struct

Serialized as an ordered list

    0. [NAME](serde://newtype_struct/NAME)
    1. VALUE

### Newtype Variant

Serialized as an ordered list

    0. [NAME](serde://newtype_variant/NAME/VARIANT)
    1. VALUE

### Seq

Serialized as an ordered list

Seq with a known length of 2:

    0. (Seq of length 2)[serde://seq/2]
    1. foo
        1. bar
        2. baz
    2. spam

Seq with unknown length (notice the missing path in the type URI)

    0. (Seq of unknown length)[serde://seq]
    1. spam
        1. ham
        2. ham
    2. green eggs
        2. boat

### Tuple

Serialized as an ordered list

    0. (Tuple of length 3)(serde://tuple/3)
    1. nom
    2. nom
    3. nom

### Tuple Struct

Serialized as an ordered list

    0. (Tuple struct FooBar with 3 fields)(serde://tuple_struct/FooBar/3)
    1. aaa
    2. bbb
    3. ccc

### Tuple Variant

Serialized as an ordered list

    0. (Tuple variant Enum::FooBar with 2 fields)(serde://tuple_variant/Enum/FooBar/3)
    1. f00d
    2. b4be

### Map

Serialized as an unordered list of length-2 ordered lists

The first unordered list item contains type information

    * [Map of length 2](serde://map/2)
    *
        0. key
        1. value
    *
        0. foo
        1. bar

This representation allows arbitrary nesting

    * [Map of length 1](serde://map/1)
    *
        0.
            1. this is a key
            2. wow
        1.
            3. this is the value
            4.
                * and it's deeply nested

### Struct

Serialized like a Map

    * [Struct S with 3 fields](serde://struct/S/3)
    *
        * r
        * 1
    *
        * g
        * 2
    *
        * b
        * 255

### Struct variant

Serialized like a Map

    * [Struct variant Colors::S with 3 fields](serde://struct_variant/Colors/S/3)
    *
        * r
        * 1
    *
        * g
        * 2
    *
        * b
        * 255
