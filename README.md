# 🧬 Inherit: A Rust Inheritance Library

`inherit` is a procedural macro library for Rust that allows you to **simulate** field and method inheritance between structs. While Rust doesn't support inheritance natively, this crate provides a clean and ergonomic way to compose structs and reuse field definitions and method implementations.

## ✨ Features
- Inherit fields from one or more parent structs.
- Inherit from grandparents (multi level inheritance).
- Automatically gain access to implemented methods from parent structs.
- Works seamlessly with Rust's existing type and trait system.
- Simplifies boilerplate when modeling hierarchical data structures.

## 🛠️ Usage
#### 1. Add the derive macro
Use the

```rust
#[inherit(Parent)]
```
attribute to specify one or more parent structs. The macro will:

- Copy the fields of the parent struct(s) into the child struct.
- Allow calling parent methods directly on child instances (when method names don’t overlap).

#### 2. Compose structs using inheritance

**Base struct**
```rust
struct Shape {
    x: u32,
    y: u32,
}

impl Shape {
    fn area(&self) -> u32 {
        println!("\t\t|Area of shape called|");
        0
    }
}
```

**Derived Struct: Rectangle**
```rust
#[inherit(Shape)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        println!("\t\t|Area of rectangle called|");
        self.width * self.height
    }
}
```

**Derived Struct: Square**
```rust
#[inherit(Shape)]
struct Square {
    width: u32,
}

impl Square {
    fn area(&self) -> u32 {
        println!("\t\t|Area of square called|");
        self.width * self.width
    }
}
```

**Derived Struct: Parallelogram**
```rust
struct _3DShape {
    z: u32
}

#[inherit(Rectangle, _3DShape)]
struct Parallelogram {
    angle: u32
}
```

#### 3. Use the derived structs
```rust
fn main() {
    let rect = Rectangle { x: 0, y: 0, width: 35, height: 45 };
    let sqr = Square { x: 0, y: 0, width: 35 };
    let par = Parallelogram { x: 0, y: 0, z: 45, width: 20, height: 45, angle: 45 };

    println!("Area of sqr: {}", sqr.area());
    println!("Area of par: {}", par.area());
    println!("Area of rect: {}", rect.area());
}
```

Output
```
        |Area of square called|
Area of sqr: 1225
		|Area of rectangle called|
Area of par: 900
		|Area of rectangle called|
Area of rect: 1575
```


## 🔬 How it Works
Under the hood, the `#[inherit(...)]` macro performs field composition. It injects fields from the listed parent struct(s) into the annotated struct in same order they are given and ensures that method resolution works through normal Rust method dispatch.

This enables a pattern similar to classical inheritance without relying on object-oriented paradigms or dynamic dispatch.
