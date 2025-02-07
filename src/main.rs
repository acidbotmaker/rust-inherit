use inherit_derive::inherit;

struct Shape {
    x: u32,
    y: u32,
}

#[inherit(struct Sh {
    x: u32,
    y: u32
})]
struct Rectangle {
    x: u32
}

fn main() {
    let x = Rectangle { x: 1, y: 12 };
    //  println!("Circle: {} {} {}", x.x, x.y, x.z);
    println!("Struct: {}", x.x);
}
