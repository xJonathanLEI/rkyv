#[cfg(test)]
mod tests {
    use crate::{util::alloc::*, validation::util::alloc::serialize_and_check};
    #[cfg(not(feature = "std"))]
    use alloc::{
        boxed::Box,
        collections::{BTreeMap, BTreeSet},
        rc::Rc,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    use rkyv::{
        access,
        bytecheck::CheckBytes,
        rancor::{Error, Failure},
        ser::Serializer,
        to_bytes,
        util::{serialize_into, AlignedBytes},
        validation::util::access_pos,
        Archive, Serialize,
    };
    #[cfg(feature = "std")]
    use std::rc::Rc;

    #[cfg(feature = "wasm")]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn basic_functionality() {
        // Regular serializing
        let value = Some("Hello world".to_string());
        let buf = to_bytes::<_, 256, Failure>(&value).unwrap();

        let result = access::<Option<String>, Failure>(buf.as_ref());
        result.unwrap();

        #[cfg(all(feature = "pointer_width_16", feature = "little_endian"))]
        // Synthetic archive (correct)
        let synthetic_buf = AlignedBytes([
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0u8, // padding to 2-alignment
            1u8, 0u8, // Some + padding
            0xf2u8, 0xffu8, // points 14 bytes backwards
            11u8, 0u8, // string is 11 characters long
        ]);

        #[cfg(all(feature = "pointer_width_16", feature = "big_endian"))]
        // Synthetic archive (correct)
        let synthetic_buf = AlignedBytes([
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0u8, // padding to 2-alignment
            1u8, 0u8, // Some + padding
            0xffu8, 0xf2u8, // points 14 bytes backwards
            0u8, 11u8, // string is 11 characters long
        ]);

        #[cfg(all(feature = "pointer_width_32", feature = "little_endian"))]
        // Synthetic archive (correct)
        let synthetic_buf = AlignedBytes([
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0u8, // padding to 4-alignment
            1u8, 0u8, 0u8, 0u8, // Some + padding
            0xf0u8, 0xffu8, 0xffu8, 0xffu8, // points 16 bytes backward
            11u8, 0u8, 0u8, 0u8, // string is 11 characters long
        ]);

        #[cfg(all(feature = "pointer_width_32", feature = "big_endian"))]
        // Synthetic archive (correct)
        let synthetic_buf = AlignedBytes([
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0u8, // padding to 4-alignment
            1u8, 0u8, 0u8, 0u8, // Some + padding
            0xffu8, 0xffu8, 0xffu8, 0xf0u8, // points 16 bytes backward
            0u8, 0u8, 0u8, 11u8, // string is 11 characters long
        ]);

        #[cfg(all(feature = "pointer_width_64", feature = "little_endian"))]
        // Synthetic archive (correct)
        let synthetic_buf = AlignedBytes([
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0u8, 0u8, 0u8, 0u8, 0u8, // padding to 8-alignment
            1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, // Some + padding
            // points 24 bytes backward
            0xe8u8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8,
            11u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, // string is 11 characters long
        ]);

        #[cfg(all(feature = "pointer_width_64", feature = "big_endian"))]
        // Synthetic archive (correct)
        let synthetic_buf = AlignedBytes([
            // "Hello world!!!!!" because otherwise the string will get inlined
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
            0x21, 0x21, 0x21, 0x21, 0x21, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, // Some + padding
            // points 24 bytes backward
            0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xffu8, 0xe8u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            11u8, // string is 11 characters long
        ]);

        let result =
            access::<Option<Box<[u8]>>, Failure>(synthetic_buf.as_ref());
        result.unwrap();

        // Out of bounds
        access_pos::<u32, Failure>(AlignedBytes([0, 1, 2, 3, 4]).as_ref(), 8)
            .expect_err("expected out of bounds error");
        // Overrun
        access_pos::<u32, Failure>(AlignedBytes([0, 1, 2, 3, 4]).as_ref(), 4)
            .expect_err("expected overrun error");
        // Unaligned
        access_pos::<u32, Failure>(AlignedBytes([0, 1, 2, 3, 4]).as_ref(), 1)
            .expect_err("expected unaligned error");
        // Underaligned
        access_pos::<u32, Failure>(&AlignedBytes([0, 1, 2, 3, 4])[1..], 0)
            .expect_err("expected underaligned error");
        // Undersized
        access::<u32, Failure>(&AlignedBytes([]).as_ref())
            .expect_err("expected out of bounds error");
    }

    #[cfg(feature = "pointer_width_32")]
    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn invalid_tags() {
        // Invalid archive (invalid tag)
        let synthetic_buf = AlignedBytes([
            2u8, 0u8, 0u8, 0u8, // invalid tag + padding
            8u8, 0u8, 0u8, 0u8, // points 8 bytes forward
            11u8, 0u8, 0u8, 0u8, // string is 11 characters long
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
        ]);

        let result =
            access_pos::<Option<Box<[u8]>>, Failure>(synthetic_buf.as_ref(), 0);
        result.unwrap_err();
    }

    #[cfg(feature = "pointer_width_32")]
    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn overlapping_claims() {
        // Invalid archive (overlapping claims)
        let synthetic_buf = AlignedBytes([
            // First string
            16u8, 0u8, 0u8, 0u8, // points 16 bytes forward
            11u8, 0u8, 0u8, 0u8, // string is 11 characters long
            // Second string
            8u8, 0u8, 0u8, 0u8, // points 8 bytes forward
            11u8, 0u8, 0u8, 0u8, // string is 11 characters long
            // "Hello world"
            0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64,
        ]);

        access_pos::<[Box<[u8]>; 2], Failure>(synthetic_buf.as_ref(), 0)
            .unwrap_err();
    }

    #[cfg(feature = "pointer_width_32")]
    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn cycle_detection() {
        use rkyv::rancor::Fallible;

        use rkyv::{validation::ArchiveContext, Archived};

        #[derive(Archive)]
        #[archive_attr(derive(Debug))]
        struct NodePtr(Box<Node>);

        #[allow(dead_code)]
        #[derive(Archive)]
        #[archive_attr(derive(Debug))]
        enum Node {
            Nil,
            Cons(#[omit_bounds] Box<Node>),
        }

        impl<S: Fallible + Serializer + ?Sized> Serialize<S> for Node {
            fn serialize(
                &self,
                serializer: &mut S,
            ) -> Result<NodeResolver, S::Error> {
                Ok(match self {
                    Node::Nil => NodeResolver::Nil,
                    Node::Cons(inner) => {
                        NodeResolver::Cons(inner.serialize(serializer)?)
                    }
                })
            }
        }

        unsafe impl<C> CheckBytes<C> for ArchivedNode
        where
            C: Fallible + ArchiveContext + ?Sized,
            C::Error: Error,
        {
            unsafe fn check_bytes(
                value: *const Self,
                context: &mut C,
            ) -> Result<(), C::Error> {
                let bytes = value.cast::<u8>();
                let tag = *bytes;
                match tag {
                    0 => (),
                    1 => {
                        <Archived<Box<Node>> as CheckBytes<C>>::check_bytes(
                            bytes.add(4).cast(),
                            context,
                        )?;
                    }
                    _ => panic!(),
                }
                Ok(())
            }
        }

        // Invalid archive (cyclic claims)
        let synthetic_buf = AlignedBytes([
            // First node
            1u8, 0u8, 0u8, 0u8, // Cons
            4u8, 0u8, 0u8, 0u8, // Node is 4 bytes forward
            // Second string
            1u8, 0u8, 0u8, 0u8, // Cons
            244u8, 255u8, 255u8, 255u8, // Node is 12 bytes back
        ]);

        access_pos::<Node, Failure>(synthetic_buf.as_ref(), 0).unwrap_err();
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn derive_unit_struct() {
        #[derive(Archive, Serialize)]
        #[archive(check_bytes)]
        struct Test;

        serialize_and_check::<_, Failure>(&Test);
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn derive_struct() {
        #[derive(Archive, Serialize)]
        #[archive(check_bytes)]
        struct Test {
            a: u32,
            b: String,
            c: Box<Vec<String>>,
        }

        serialize_and_check::<_, Failure>(&Test {
            a: 42,
            b: "hello world".to_string(),
            c: Box::new(vec!["yes".to_string(), "no".to_string()]),
        });
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn derive_tuple_struct() {
        #[derive(Archive, Serialize)]
        #[archive(check_bytes)]
        struct Test(u32, String, Box<Vec<String>>);

        serialize_and_check::<_, Failure>(&Test(
            42,
            "hello world".to_string(),
            Box::new(vec!["yes".to_string(), "no".to_string()]),
        ));
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn derive_enum() {
        #[derive(Archive, Serialize)]
        #[archive(check_bytes)]
        enum Test {
            A(u32),
            B(String),
            C(Box<Vec<String>>),
        }

        serialize_and_check::<_, Failure>(&Test::A(42));
        serialize_and_check::<_, Failure>(&Test::B("hello world".to_string()));
        serialize_and_check::<_, Failure>(&Test::C(Box::new(vec![
            "yes".to_string(),
            "no".to_string(),
        ])));
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn recursive_type() {
        #[derive(Archive, Serialize)]
        // The derive macros don't apply the right bounds from Box so we have to manually specify
        // what bounds to apply
        #[archive(
            serialize_bounds(__S: Serializer),
            deserialize_bounds(__D: Deserializer),
        )]
        #[archive(check_bytes)]
        #[archive_attr(check_bytes(
            bounds(__C: ::rkyv::validation::ArchiveContext)
        ))]
        enum Node {
            Nil,
            Cons(
                #[omit_bounds]
                #[archive_attr(omit_bounds)]
                Box<Node>,
            ),
        }

        serialize_and_check::<_, Failure>(&Node::Cons(Box::new(Node::Cons(
            Box::new(Node::Nil),
        ))));
    }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn check_shared_ptr() {
        #[derive(Archive, Serialize, Eq, PartialEq)]
        #[archive(check_bytes)]
        struct Test {
            a: Rc<u32>,
            b: Rc<u32>,
        }

        let shared = Rc::new(10);
        let value = Test {
            a: shared.clone(),
            b: shared.clone(),
        };

        let serializer = serialize_into::<_, _, Failure>(
            &value,
            DefaultSerializer::default(),
        )
        .unwrap();
        let buf = serializer.into_serializer().into_inner();

        access::<Test, Failure>(buf.as_ref()).unwrap();
    }

    // TODO: re-enable after btreemap validation is fixed
    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn check_b_tree() {
    //     let mut value = BTreeMap::new();
    //     value.insert("foo".to_string(), 10);
    //     value.insert("bar".to_string(), 20);
    //     value.insert("baz".to_string(), 40);
    //     value.insert("bat".to_string(), 80);

    //     let mut serializer = DefaultSerializer::default();
    //     serializer.serialize_value(&value).unwrap();
    //     let buf = serializer.into_serializer().into_inner();

    //     access::<BTreeMap<String, i32>, Failure>(buf.as_ref()).unwrap();
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn check_invalid_b_tree_set() {
    //     let data = AlignedBytes([
    //         0, 0, 0, 0, 253, 6, 239, 6, 255, 255, 255, 252, 0, 0, 0, 0, 0, 0,
    //         0, 0, 1, 0, 0, 5, 0, 0, 0, 0, 240, 255, 255, 255, 1, 128, 0, 249,
    //         220, 255, 255, 255, 4, 0, 0, 96, 0, 0, 0, 249, 232, 255, 255, 255,
    //     ]);

    //     rkyv::from_bytes::<BTreeSet<u8>, Failure>(&data.0).unwrap_err();

    //     let data = AlignedBytes([
    //         1, 29, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 253, 0, 0, 116, 255, 255, 40,
    //         0, 8, 0, 0, 0, 236, 255, 255, 255, 1, 128, 72, 0, 220, 255, 255,
    //         255, 236, 255, 255, 255, 0, 0, 0, 0, 32, 0, 255, 254, 255, 0, 94,
    //         2, 33, 0, 0, 0, 0, 0, 0, 0, 61, 1, 38, 0, 0, 32, 0, 255, 255, 1, 0,
    //         1, 255, 255, 0, 184, 4, 0, 28, 0, 8, 0, 2, 142, 255, 255, 255, 3,
    //         1, 255, 251, 0, 184, 255, 255, 255,
    //     ]);

    //     rkyv::from_bytes::<BTreeSet<Box<u8>>, Failure>(&data.0).unwrap_err();
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn check_empty_b_tree() {
    //     let value = BTreeMap::<u8, ()>::new();

    //     let mut serializer = DefaultSerializer::default();
    //     serializer.serialize_value(&value).unwrap();
    //     let buf = serializer.into_serializer().into_inner();

    //     access::<BTreeMap<u8, ()>, Failure>(buf.as_ref()).unwrap();
    // }

    // #[test]
    // // This test is unfortunately too slow to run through miri
    // #[cfg_attr(miri, ignore)]
    // // This test creates structures too big to fit in 16-bit offsets
    // #[cfg(not(feature = "pointer_width_16"))]
    // fn check_b_tree_large() {
    //     let mut value = BTreeMap::new();
    //     for i in 0..100_000 {
    //         value.insert(i.to_string(), i);
    //     }

    //     let mut serializer = DefaultSerializer::default();
    //     serializer.serialize_value(&value).unwrap();
    //     let buf = serializer.into_serializer().into_inner();

    //     access::<BTreeMap<String, i32>, Failure>(buf.as_ref()).unwrap();
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn b_tree_struct_member() {
    //     #[derive(Archive, Serialize, Deserialize, Debug, Default)]
    //     #[archive(check_bytes)]
    //     pub struct MyType {
    //         pub some_list: BTreeMap<String, Vec<f32>>,
    //         pub values: Vec<f32>,
    //     }

    //     let mut value = MyType::default();

    //     value
    //         .some_list
    //         .entry("Asdf".to_string())
    //         .and_modify(|e| e.push(1.0))
    //         .or_insert_with(|| vec![2.0]);

    //     let serializer = serialize_into::<_, _, Failure>(
    //         &value,
    //         DefaultSerializer::default(),
    //     ).unwrap();
    //     let buf = serializer.into_serializer().into_inner();

    //     let _ = from_bytes::<MyType, Failure>(&buf).unwrap();
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn check_valid_durations() {
    //     use core::time::Duration;

    //     access::<Duration, Failure>(&[0xFF, 16]).unwrap_err();
    // }

    // #[test]
    // #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    // fn check_invalid_btreemap() {
    //     let data = AlignedBytes([
    //         0, 0, 0, 0, 0, 0, 0, 0, 0, 0x30, 0, 0x00, 0x00, 0x00, 0x0c, 0xa5,
    //         0xf0, 0xff, 0xff, 0xff,
    //     ]);
    //     rkyv::from_bytes::<BTreeMap<u8, Box<u8>>, Failure>(&data.0).unwrap_err();
    // }

    #[test]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn check_invalid_string() {
        let data = AlignedBytes([0x10; 16]);
        rkyv::from_bytes::<String, Failure>(&data.0).unwrap_err();
    }
}
