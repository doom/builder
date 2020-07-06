use builder_derive::Builder;

#[derive(Builder, Debug)]
struct Something {
    field1: u32,
    field2: String,
    #[builder(ignore)]
    ignored_field: u32,
}

impl Default for Something {
    fn default() -> Self {
        Self {
            field1: u32::default(),
            field2: "a default".to_string(),
            ignored_field: u32::default(),
        }
    }
}

fn main() {
    let x = Something::default();
    println!("{:?}", &x);

    let x = x.set_field1(1).set_field2("lalala".to_string());
    println!("{:?}", &x);
}
