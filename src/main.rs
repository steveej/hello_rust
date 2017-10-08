extern crate invsearch;

extern crate libc;

// struct Invoice {
//     date: String,
//     number: String,
//     sirname: String,
//     name: String,
// }

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

        // Read the file contents into a string, returns `io::Result<usize>`

        let file_content = match ::invsearch::read_to_string(&mut file) {
            Err(e) => {
                eprintln!("{}", e);
                ::std::process::exit(::libc::EINVAL)
            }
            Ok(s) => s,
        };

        println!("{}", file_content);
    }
}