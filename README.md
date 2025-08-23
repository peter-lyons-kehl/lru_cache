# LRU cache

Configurable, efficient, simple and safe LRU (Least Recently Used) cache.

## Configurable at zero cost

**Behavior** is configurable at compile time: with const generics. That means that any extra
behavior (like recycling sequential indices) is compiled only when you opt for it.

That is intentionally not done through crate features. This way you can have instances with
different behavior in the same consumer crate(s). That also prevents any build conflicts between
consumer crates that would otherwise use conflicting features.

## Safe

`lib.rs` has `#![forbid(unsafe_code)]`.
