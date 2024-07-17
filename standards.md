# Standards

* create buffer id's as so:
```rust
    use std::ptr;
    let id = unsafe {
        let mut id = 0;
        gl::function(..., ptr::addr_of_mut!(id), ...);
        gl::other(..., id, ...);
        id
    }
```