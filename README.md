# LRU cache

Configurable, efficient, simple and safe LRU (Least Recently Used) cache.

## Configurable at zero cost

**Behavior** is configurable at compile time: with const generics. That means that any extra
behavior (like recycling sequential indices) is compiled only when you opt for it.

That is intentionally not done through crate features. This way you can have instances with
different behavior in the same consumer crate(s). That also prevents any build conflicts between
consumer crates that would otherwise use conflicting features.

## Safe

`lib.rs` starts with `#![forbid(unsafe_code)]`.

## See also

- https://crates.io/crates/lru-cache
- https://crates.io/crates/hashlru
- https://crates.io/crates/lrumap
- https://crates.io/keywords/lru and https://github.com/khonsulabs/
- https://crates.io/crates/hashlink
- https://crates.io/crates/clru
- https://crates.io/crates/uluru
- https://crates.io/crates/lazy-lru
- https://crates.io/crates/lru
- https://crates.io/crates/intrusive-lru-cache
