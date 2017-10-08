#[macro_use]
extern crate invsearch;

extern crate libc;
extern crate assert_cli;

mod integration {
    use assert_cli;

    #[test]
    fn called_without_args() {
        assert_cli::Assert::main_binary()
            .fails_with(::libc::EINVAL)
            .and()
            .stderr()
            .contains(MISSING_ARGUMENTS_ERR!())
            .unwrap();
    }

    #[test]
    fn called_with_invalid_file() {
        let args = &["/not/found"];
        assert_cli::Assert::main_binary()
            .with_args(args)
            .fails_with(::libc::ENOENT)
            // .and()
            // .stderr().contains(MISSING_ARGUMENTS_OUTPUT)
            .unwrap();
    }
}