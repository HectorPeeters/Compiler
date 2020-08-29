trait Generator {
    fn new() -> Self;
    fn test(&mut self);
}

struct X86Generator {

}

impl Generator for X86Generator {
    fn new() -> Self {
        X86Generator {

        }
    }

    fn test(&mut self) {
        println!("Test");
    }
}

fn main() {
    X86Generator::new().test();
}
