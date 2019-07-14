mod types {
    use inherent::inherent;

    trait Trait {
        fn f<T: ?Sized>(self);
    }

    pub struct Struct;

    #[inherent(pub)]
    impl Trait for Struct {
        fn f<T: ?Sized>(self) {}
    }
}

#[test]
fn test() {
    // types::Trait is not in scope.
    types::Struct.f::<str>();
}
