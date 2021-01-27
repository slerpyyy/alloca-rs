# alloca-rs

Mostly safe wrapper for `alloca` in Rust.


This crate uses Rust lifetime system to ensure that stack allocated memory will not be used after function return, but it does not make any guarantee about memory that is turned into raw pointer and stored somewhere else. 

# Example

```rust
fn main() {
    alloca::with_alloca(128, /* how much bytes we want to allocate */
        |memory: &mut [u8] /* dynamically stack allocated slice itself */|
     {
            assert!(memory.len() == 128);
            memory[42] = 42;
            println!("{}",memory[42]);
    });
}
```