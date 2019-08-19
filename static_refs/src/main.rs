/// Examples for retrieving static references for runtime variables
///
/// Output:
/// 0x555bf3661ca0
/// 0x555bf3661ce0
/// 0x555bf3661d20
/// lazy_static emits the same address as it uses a static variable
/// 0x555bf1b4c050
/// 0x555bf1b4c050
/// 0x555bf1b4c050
/// the manual implementation also uses a static variable
/// 0x555bf3661d80
/// 0x555bf3661d80
/// 0x555bf3661d80

#[macro_use]
extern crate lazy_static;

fn get_leaked_ref() -> &'static String {
    Box::leak(Box::new("hello".to_string()))
}

fn get_lazy_ref() -> &'static String {
    lazy_static! {
        static ref HELLO: String = "hello".to_string();
    }

    &HELLO
}

// this is the approach used by lazy_static crate
// see: https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
fn get_singleton_ref() -> &'static String {
    use std::mem;
    use std::sync::{Once, ONCE_INIT};

    static mut SINGLETON: *const String = 0 as *const String;
    static ONCE: Once = ONCE_INIT;

    unsafe {
        ONCE.call_once(|| SINGLETON = mem::transmute(Box::new("hello".to_string())));
        &(*SINGLETON)
    }
}

fn main() {
    println!("the leak approach allocates every time");
    for _ in 0..3 {
        println!("{:p}", get_leaked_ref());
    }

    println!("lazy_static emits the same address as it uses a static variable");
    for _ in 0..3 {
        println!("{:p}", get_lazy_ref());
    }

    println!("the manual implementation also uses a static variable");
    for _ in 0..3 {
        println!("{:p}", get_singleton_ref());
    }
}
