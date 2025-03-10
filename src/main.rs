use inherit_derive::inherit;

struct Shape {
    x: u32,
    y: u32,
}

struct _3DShape {
    z: u32
}

#[inherit(Shape)]
#[derive(Debug)]
struct Rectangle{
    width: u32,
    height: u32,
}

#[inherit(Shape)]
struct Square {
    width: u32,
}

#[inherit(Rectangle, _3DShape)] // TODO: Multi-level inheritance is pending
struct Parallelogram {
    angle: u32
}


fn main() {
    let rect = Rectangle { x: 0, y: 0, width: 35, height: 45 };
    let sqr = Square {x: 0, y: 0, width: 35};
    let par = Parallelogram { x: 0, y: 0, width: 35, height: 45, angle: 45, z: 45 };

    println!("{:#?} {:#?} {:#?}", rect, sqr, par);
}
