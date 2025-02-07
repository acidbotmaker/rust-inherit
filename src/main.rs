use inherit_derive::inherit;

struct Shape {
    x: u32,
    y: u32,
}

struct _3DShape {
    z: u32
}

#[inherit(Shape, _3DShape)]
struct Rectangle;

// #[inherit(Rectangle)]
// struct Square{
//     w: f32
// }

fn main() {
    let x = Rectangle { x: 1, y: 12, z: 35 };

    // let square = Square {z: 23, w: 32.0};
    // println!("{:#?} {:#?}", x, square);
    println!("{:#?}", x);
}
