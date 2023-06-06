use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fs;

#[proc_macro]
pub fn integration_tests(_: TokenStream) -> TokenStream {
    let tests = fs::read_dir("tests/src")
        .unwrap()
        .map(|f| {
            let test_path = f.unwrap().path();
            let test_name = format_ident!("r#{}", test_path.file_stem().unwrap().to_str().unwrap());
            let test_path = test_path.to_str().unwrap();
            quote! {
                #[test]
                fn #test_name() {
                    ::test_utils::run_test(#test_path, ::test_utils::Mode::Compare);
                }
            }
        })
        .map(|ts| TokenStream::from(ts));

    TokenStream::from_iter(tests)
}
