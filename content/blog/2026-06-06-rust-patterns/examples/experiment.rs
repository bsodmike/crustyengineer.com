use std::fmt::{Display, Formatter};

trait Drawable: Display {
    fn draw(&self);
    fn area(&self) -> f64;
}

trait LoggableDrawables: Drawable + Display {}

struct Circle {
    radius: f64,
}

impl LoggableDrawables for Circle {}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle r={}", self.radius);
    }
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Display for Circle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Circle radius: {}", self.radius))
    }
}

struct Square {
    side: f64,
}

impl LoggableDrawables for Square {}

impl Drawable for Square {
    fn draw(&self) {
        println!("Drawing square s={}", self.side);
    }
    fn area(&self) -> f64 {
        self.side * self.side
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Square side: {}", self.side))
    }
}

fn log_all(items: &[Box<dyn LoggableDrawables>]) {
    for item in items {
        println!("{item}"); // vtable dispatch
    }
}

fn main() {
    let shapes: Vec<Box<dyn LoggableDrawables>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Square { side: 3.0 }),
    ];

    let _ = log_all(&shapes[..]);

    // Each element is a fat pointer: (data_ptr, vtable_ptr)
    // The vtable for Circle and Square are DIFFERENT
    for shape in &shapes {
        shape.draw(); // vtable dispatch → Circle::draw or Square::draw
        println!("  area = {:.2}", shape.area());
    }

    // Size comparison:
    println!("size_of::<&Circle>()        = {}", size_of::<&Circle>());
    // → 8 bytes (one pointer — the compiler knows the type)
    println!(
        "size_of::<&dyn Drawable>()  = {}",
        size_of::<&dyn Drawable>()
    );
    // → 16 bytes (data_ptr + vtable_ptr)
}
