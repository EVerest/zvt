use zvt::{encoding, length, Tag, ZVTError, Zvt, ZvtSerializer, ZvtSerializerImpl};

#[test]
#[rustfmt::skip]
fn test_zvt_serializer_impl() {
    type Input = u16;
    // Test without length and tag.
    assert_eq!(<Input as ZvtSerializerImpl>::serialize_tagged(&12, None), [12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[12, 0], None).unwrap().0, 12);

    // Tests with length and custom encoding.
    assert_eq!(<Input as ZvtSerializerImpl<length::Tlv>>::serialize_tagged(&12, None), [2, 12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl<length::Tlv>>::deserialize_tagged(&[2, 12, 0], None).unwrap().0, 12);
    assert_eq!(<Input as ZvtSerializerImpl<length::Adpu>>::serialize_tagged(&12, None), [2, 12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl<length::Adpu>>::deserialize_tagged(&[2, 12, 0], None).unwrap().0, 12);
    assert_eq!(<Input as ZvtSerializerImpl<length::Adpu, encoding::Bcd>>::serialize_tagged(&12, None), [1, 0x12]);
    assert_eq!(<Input as ZvtSerializerImpl<length::Adpu, encoding::Bcd>>::deserialize_tagged(&[1, 0x12], None).unwrap().0, 12);

    // Test with length and tags.
    assert_eq!(<Input as ZvtSerializerImpl>::serialize_tagged(&12, Some(Tag(11))), [11, 12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[11, 12, 0], Some(Tag(11))).unwrap().0, 12);
    assert_eq!(<Input as ZvtSerializerImpl<length::Tlv>>::serialize_tagged(&12, Some(Tag(11))), [11, 2, 12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl<length::Tlv>>::deserialize_tagged(&[11, 2, 12, 0], Some(Tag(11))).unwrap().0, 12);
    assert_eq!(<Input as ZvtSerializerImpl<length::Tlv, encoding::BigEndian, encoding::BigEndian>>::serialize_tagged(&12, Some(Tag(11))), [0, 11, 2, 0, 12]);
    assert_eq!(<Input as ZvtSerializerImpl<length::Tlv, encoding::BigEndian, encoding::BigEndian>>::deserialize_tagged(&[0, 11, 2, 0, 12], Some(Tag(11))).unwrap().0, 12);

    // Test failures
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[1], None), Err(ZVTError::IncompleteData));
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[12, 12, 0], Some(Tag(11))), Err(ZVTError::WrongTag(Tag(12))));
}

#[test]
#[rustfmt::skip]
fn test_zvt_serializer_impl_option() {
    // Tests the optional logic of the ZvtSerializerImpl.
    type Input = Option<u16>;
    // Test without a tag.
    assert_eq!(<Input as ZvtSerializerImpl>::serialize_tagged(&Some(12), None), [12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[12, 0], None).unwrap().0, Some(12));

    assert_eq!(<Input as ZvtSerializerImpl>::serialize_tagged(&None, None), []);
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[0], None).unwrap().0, None);

    // Test with a tag.
    assert_eq!(<Input as ZvtSerializerImpl>::serialize_tagged(&Some(12), Some(Tag(11))), [11, 12, 0]);
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[11, 12, 0], Some(Tag(11))).unwrap().0, Some(12));

    assert_eq!(<Input as ZvtSerializerImpl>::serialize_tagged(&None, Some(Tag(11))), []);
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[], Some(Tag(11))), Err(ZVTError::IncompleteData));
    assert_eq!(<Input as ZvtSerializerImpl>::deserialize_tagged(&[0], Some(Tag(11))), Err(ZVTError::WrongTag(Tag(0))));
}

#[test]
fn test_command() {
    #[derive(Zvt, PartialEq, Debug)]
    #[zvt_control_field(class = 13, instr = 12)]
    struct Bar {
        a: u8,

        #[zvt_bmp(number = 0x3, encoding = encoding::BigEndian)]
        b: Option<u32>,

        #[zvt_bmp(number = 0x1)]
        c: u16,

        #[zvt_bmp(number = 0x2)]
        d: Option<u32>,
    }

    // Some legitimate values.
    let values = [
        (
            Bar {
                a: 1,
                b: Some(2),
                c: 3,
                d: None,
            },
            vec![13, 12, 9, 1, 3, 0, 0, 0, 2, 1, 3, 0],
        ),
        (
            Bar {
                a: 1,
                b: None,
                c: 3,
                d: Some(4),
            },
            vec![13, 12, 9, 1, 1, 3, 0, 2, 4, 0, 0, 0],
        ),
    ];

    for (input, bytes) in values {
        assert_eq!(input.zvt_serialize(), bytes);
        assert_eq!(Bar::zvt_deserialize(&bytes).unwrap().0, input);
    }

    // The tagged values but not in order.
    let values = [
        vec![13, 12, 14, 1, 2, 4, 0, 0, 0, 1, 3, 0, 3, 0, 0, 0, 2],
        vec![13, 12, 14, 1, 2, 4, 0, 0, 0, 3, 0, 0, 0, 2, 1, 3, 0],
    ];

    let expected = Bar {
        a: 1,
        b: Some(2),
        c: 3,
        d: Some(4),
    };
    for bytes in values {
        assert_eq!(Bar::zvt_deserialize(&bytes).unwrap().0, expected);
    }

    // Missing required field c
    let bytes = vec![13, 12, 11, 1, 2, 4, 0, 0, 0, 3, 0, 0, 0, 2];
    assert_eq!(
        Bar::zvt_deserialize(&bytes),
        Err(ZVTError::MissingRequiredTags(vec![Tag(0x1)]))
    );

    // Duplicate field b
    let bytes = vec![13, 12, 14, 1, 3, 0, 0, 0, 2, 1, 3, 0, 3, 0, 0, 0, 2];
    assert_eq!(
        Bar::zvt_deserialize(&bytes),
        Err(ZVTError::DuplicateTag(Tag(3)))
    );
}

#[test]
fn test_encodings() {
    #[derive(Zvt, Debug, PartialEq)]
    struct Foo {
        #[zvt_bmp(encoding = encoding::BigEndian)]
        a: u16,

        #[zvt_bmp(length = length::Tlv, encoding = encoding::Bcd)]
        b: u32,

        #[zvt_bmp(encoding = encoding::Hex)]
        c: String,
    }

    let input = Foo {
        a: 1,
        b: 2,
        c: "ffde".to_string(),
    };

    let expected_bytes = vec![0, 1, 1, 2, 0xff, 0xde];
    let bytes = input.zvt_serialize();
    assert_eq!(bytes, expected_bytes);
    assert_eq!(Foo::zvt_deserialize(&expected_bytes).unwrap().0, input);
}

#[test]
fn test_named_struct() {
    #[derive(Zvt, Debug, PartialEq)]
    struct MyNamedStruct {
        x: u8,
        y: u32,
    }
    let d = MyNamedStruct { x: 1, y: 2 };
    let expected_bytes = vec![1, 2, 0, 0, 0];
    assert_eq!(expected_bytes, d.zvt_serialize());
    let deser = MyNamedStruct::zvt_deserialize(&expected_bytes).unwrap();
    assert_eq!(deser.0, d);
    assert!(deser.1.is_empty())
}

#[test]
fn test_foo() {
    #[derive(Zvt, PartialEq, Debug)]
    struct Foo {
        #[zvt_bmp(length = length::Fixed<2>, encoding = encoding::Bcd)]
        a: usize,

        #[zvt_bmp(number = 0x13, length = length::Fixed<4>, encoding = encoding::Bcd)]
        b: Option<usize>,
    }
    let f = Foo { a: 1, b: None };
    let bytes = f.zvt_serialize();
    let o = Foo::zvt_deserialize(bytes.as_slice()).unwrap();
    assert_eq!(f, o.0);
}

#[test]
fn test_nested() {
    #[derive(Zvt, PartialEq, Debug)]
    struct Inner {
        a: u8,
        #[zvt_bmp(number = 0x12)]
        b: Option<u8>,
    }

    #[derive(Zvt, PartialEq, Debug)]
    struct Outer {
        a: u16,
        #[zvt_tlv(tag = 0x12)]
        b: Option<Inner>,
    }

    let f = Outer {
        a: 1,
        b: Some(Inner { a: 2, b: Some(3) }),
    };
    let bytes = f.zvt_serialize();
    let o = Outer::zvt_deserialize(bytes.as_slice()).unwrap();
    assert_eq!(f, o.0);
}

#[test]
fn test_tagged_but_optional_but_still_tagged() {
    #[derive(Zvt, PartialEq, Debug)]
    struct Foo {
        #[zvt_bmp(number = 0x11)]
        a: u16,
        #[zvt_bmp(number = 0x12)]
        b: Option<u16>,
    }

    // The simple case
    for f in [Foo { a: 1, b: Some(2) }, Foo { a: 1, b: None }] {
        let bytes = f.zvt_serialize();
        let o = Foo::zvt_deserialize(bytes.as_slice()).unwrap();
        assert_eq!(f, o.0);
    }

    // Now check the case where the required field is missing.
    for data in [vec![], vec![0x12, 0, 0]] {
        assert_eq!(
            Foo::zvt_deserialize(&data),
            Err(ZVTError::MissingRequiredTags(vec![Tag(0x11)]))
        );
    }
}
