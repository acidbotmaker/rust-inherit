use inherit_derive::inherit;

struct Shape {
    x: u32,
    y: u32,
}

impl Shape {
    fn area() -> u32 {
        0
    }
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

impl Rectangle {
    fn area(&self) -> u32 {
        return self.width * self.height;
    }
}

#[inherit(Shape)]
struct Square {
    width: u32,
}

impl Square {
    fn area(&self) -> u32 {
        return self.width * self.width;
    }
}

#[inherit(Rectangle, _3DShape)] // Multi-level inheritance is pending
struct Parallelogram {
    angle: u32
}


fn main() {
    let rect = Rectangle { x: 0, y: 0, width: 35, height: 45 };
    let sqr = Square {x: 0, y: 0, width: 35};
    let par = Parallelogram { x: 0, y: 0, width: 20, height: 45, angle: 45, z: 45 };

    println!("{:#?} {:#?} {:#?}", rect, sqr, par);
    println!("Area of sqr: {}", sqr.area());
    println!("Area of par: {}", par.area());
    println!("Area of rect: {}", rect.area());
}
