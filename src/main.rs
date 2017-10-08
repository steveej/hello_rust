extern crate invsearch;
extern crate libc;

fn main() {
    // Receive command line arguments
    let args = match ::invsearch::get_args() {
        Err(e) => {
            eprintln!("{}", e);
            ::std::process::exit(::libc::EINVAL)
        }
        Ok(a) => a,
    };

    for arg in args[1..].iter() {
        // Create a path to the desired file


        let mut file = match ::invsearch::open_file(arg) {
            Err((string, code)) => {
                eprintln!("{}", string);
                ::std::process::exit(code)
            }
            Ok(f) => f,
        };

        match ::invsearch::parse_invoice(&mut file) {
            Ok(inv) => println!("{:?}", inv),
            Err(e) => eprintln!("'{}': {}", arg, e),
        };

        // Read the file contents into a string, returns `io::Result<usize>`

        // let file_content = match ::invsearch::read_to_string(&mut file) {
        //     Err(e) => {
        //         eprintln!("{}", e);
        //         ::std::process::exit(::libc::EINVAL)
        //     }
        //     Ok(s) => s,
        // };
    }
}