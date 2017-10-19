extern crate invsearch;

extern crate libc;
extern crate assert_cli;

/// Can currently be tested using:
///
/// ```bash
/// for inv in tests/assets/invoice_*; do curl -X POST 127.0.0.1:3000/invoice/parse --data-binary @${inv}; printf n; done
/// ```
mod integration {
    /* TODO: migrate this to a client

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
    */
}