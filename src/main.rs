use inherit_derive::inherit;

struct Shape {
    x: u32,
    y: u32,
}

#[inherit(Shape)]
struct Rectangle {
    z: u32,
}

fn main() {
    let x = Rectangle { x: 1, y: 12, z: 35 };
    println!("{:#?}", x);
}
