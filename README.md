# TypeSift

A Rust derive macro that walks structs, enums, collections and maps to collect every value of a given type.

## Usage

```rust
use typesift::TypeSift;

#[derive(Debug, PartialEq, TypeSift)]
struct UserId(u64);

#[derive(TypeSift)]
struct Group {
    name: String,
    admin: UserId,
    members: Vec<UserId>,
}

let group = Group {
    name: "Core".to_string(),
    admin: UserId(7),
    members: vec![UserId(12), UserId(9)],
};

// Get references to all values of type `UserId` inside of `group`
let user_ids = group.sift::<UserId>();
assert_eq!(user_ids, [&UserId(7), &UserId(12), &UserId(9)]);
```

## Motivation

Big API responses tend to be graphs of DTOs that point at other resources by id.
Before responding you want those resources loaded.
Collecting the ids normally means a function that walks every field by hand, and that function may quietly go stale when a new field is added.

Typesift reduces the amount of code that needs to be written and eliminates this concern.


## Integration with third-party crates

If you don't own a type, the orphan rule stops it from implementing the trait.

### Feature flags

Support for types from some popular crates can be enabled with the corresponding crate feature: `bytes`, `chrono`, `enum-map`, `http`, `indexmap`, `rust_decimal`, `serde_json`, `smallvec`, `time`, `url`, `uuid`.

Example:
```toml
typesift = { version = "0.1", features = ["uuid"] }
```

### Typesift attributes

If the crate you're interested in is not supported, typesift offers you the following macro attributes to overcome the issue:

- `#[typesift(leaf)]` finds the field but doesn't look inside it. This is what you want for ids, timestamps and other opaque values.
- `#[typesift(skip)]` leaves it out of the search entirely. Also the escape hatch for `RefCell`, `Mutex` and friends, which can't lend out their contents for as long as the search needs them.
- `#[typesift(with = path)]` hands the field to a function of yours, which decides what the search sees and in what order.


## Limitations

Everything involved has to be `'static`, so the derive rejects types with lifetime parameters, though `&'static T` fields are fine.
Interior mutability isn't supported; skip those fields.


## License

MIT © [Serhii Potapov](https://www.greyblake.com)

