# builder

proc_macro experiment to automatically implement the builder pattern

Considering the following `struct` definition :

```rust
#[derive(Builder, Debug)]
struct Something {
    field1: u32,
    field2: String,
    #[builder(ignore)]
    ignored_field: u32,
}
```

The `Builder` macro will generate the below implementation of the builder pattern for `Something` :

```rust
impl Something {
    pub fn set_field1(mut self, value: u32) -> Self {
        self.field1 = value;
        self
    }

    pub fn set_field2(mut self, value: String) -> Self {
        self.field2 = value;
        self
    }
}

```
