fn main() {
    use std::io::prelude::*;
    use std::fs::File;

    fn foo() -> std::io::Result<()> {
        let mut f = try!(File::create("foo.txt"));
        try!(f.write_all(b"Hello, world!"));

        let mut f = try!(File::open("foo.txt"));
        let mut s = String::new();
        try!(f.read_to_string(&mut s));
        assert_eq!(s, "Hello, world!");
        Ok(())
    }

    match foo() {
        Ok(()) => println!("All good!"),
        _ => println!("Something happened"),
    }
}
