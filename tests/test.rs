mod types {
    use inherent::inherent;

    trait Trait {
        fn f<T: ?Sized>(self);
        // A default method
        fn g(&self) {}
    }

    pub struct Struct;

    #[inherent]
    impl Trait for Struct {
        pub fn f<T: ?Sized>(self) {}
        #[rustfmt::skip] // https://github.com/rust-lang/rustfmt/issues/4960
        pub fn g(&self);
    }
}

#[test]
fn test() {
    // types::Trait is not in scope.
    let s = types::Struct;
    s.g();
    s.f::<str>();
}
