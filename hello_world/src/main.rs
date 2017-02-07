fn main() {
    println!("{} {}", say_hello("me"), say_hello("you"));
}

pub fn say_hello(name: &str) -> String {
    let message = format!("hello, {}!", name);
    message
}
