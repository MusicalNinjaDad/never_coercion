#![feature(never_type)]
#![allow(dead_code)]

#[derive(Debug)]
struct Foo<T: HasAssocType> {
    data: T::AssocType,
}

trait HasAssocType: Sized {
    type AssocType;
}

impl HasAssocType for ! {
    type AssocType = [u8; 0];
}

impl HasAssocType for u8 {
    type AssocType = [u8; 1];
}

// // error[E0308]: mismatched types
// //   --> examples/generic.rs:43:20
// //    |
// // 38 |     fn map<U, F>(self, f: F) -> Foo<U>
// //    |            - found this type parameter
// // ...
// // 43 |         Foo{ data: f(self.data) }
// //    |                    ^^^^^^^^^^^^ expected associated type, found type parameter `U`
// //    |
// //    = note: expected associated type `<U as HasAssocType>::AssocType`
// //                found type parameter `U`
// // help: consider further restricting this bound
// //    |
// // 40 |         U: HasAssocType<AssocType = U>,
// //    |                        +++++++++++++++
//
// impl<T: HasAssocType> Foo<T> {
//     fn map<U, F>(self, f: F) -> Foo<U>
//     where
//         U: HasAssocType,
//         F: FnOnce(T) -> U,
//     {
//         Foo{ data: f(self.data) }
//     }
// }

fn main() {
    let never_foo = Foo::<!> { data: [] };

    let u8_foo = Foo::<u8> { data: [1] };

    println!("{never_foo:?}, {u8_foo:?}");
}
