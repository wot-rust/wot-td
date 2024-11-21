use alloc::vec::Vec;
use core::fmt::{self, Display};
use serde::ser::{self, Impossible, Serialize, SerializeMap, Serializer};

macro_rules! tri {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
}

#[cfg(any(feature = "std", feature = "alloc"))]
use self::content::{Content, ContentSerializer};

enum Unsupported {
    Boolean,
    Integer,
    Float,
    Char,
    String,
    ByteArray,
    #[cfg(any(feature = "std", feature = "alloc"))]
    UnitStruct,
    Sequence,
    Tuple,
    TupleStruct,
    Enum,
}

impl Display for Unsupported {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Unsupported::Boolean => formatter.write_str("a boolean"),
            Unsupported::Integer => formatter.write_str("an integer"),
            Unsupported::Float => formatter.write_str("a float"),
            Unsupported::Char => formatter.write_str("a char"),
            Unsupported::String => formatter.write_str("a string"),
            Unsupported::ByteArray => formatter.write_str("a byte array"),
            #[cfg(any(feature = "std", feature = "alloc"))]
            Unsupported::UnitStruct => formatter.write_str("unit struct"),
            Unsupported::Sequence => formatter.write_str("a sequence"),
            Unsupported::Tuple => formatter.write_str("a tuple"),
            Unsupported::TupleStruct => formatter.write_str("a tuple struct"),
            Unsupported::Enum => formatter.write_str("an enum"),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
mod content {
    use alloc::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};
    use core::marker::PhantomData;
    use serde::ser::{self, Serialize, Serializer};

    pub enum Content {
        Bool(bool),

        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),

        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),

        F32(f32),
        F64(f64),

        Char(char),
        String(String),
        Bytes(Vec<u8>),

        None,
        Some(Box<Content>),

        Unit,
        UnitStruct(&'static str),
        UnitVariant(&'static str, u32, &'static str),
        NewtypeStruct(&'static str, Box<Content>),
        NewtypeVariant(&'static str, u32, &'static str, Box<Content>),

        Seq(Vec<Content>),
        Tuple(Vec<Content>),
        TupleStruct(&'static str, Vec<Content>),
        TupleVariant(&'static str, u32, &'static str, Vec<Content>),
        Map(Vec<(Content, Content)>),
        Struct(&'static str, Vec<(&'static str, Content)>),
        StructVariant(
            &'static str,
            u32,
            &'static str,
            Vec<(&'static str, Content)>,
        ),
    }

    impl Serialize for Content {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match *self {
                Content::Bool(b) => serializer.serialize_bool(b),
                Content::U8(u) => serializer.serialize_u8(u),
                Content::U16(u) => serializer.serialize_u16(u),
                Content::U32(u) => serializer.serialize_u32(u),
                Content::U64(u) => serializer.serialize_u64(u),
                Content::I8(i) => serializer.serialize_i8(i),
                Content::I16(i) => serializer.serialize_i16(i),
                Content::I32(i) => serializer.serialize_i32(i),
                Content::I64(i) => serializer.serialize_i64(i),
                Content::F32(f) => serializer.serialize_f32(f),
                Content::F64(f) => serializer.serialize_f64(f),
                Content::Char(c) => serializer.serialize_char(c),
                Content::String(ref s) => serializer.serialize_str(s),
                Content::Bytes(ref b) => serializer.serialize_bytes(b),
                Content::None => serializer.serialize_none(),
                Content::Some(ref c) => serializer.serialize_some(&**c),
                Content::Unit => serializer.serialize_unit(),
                Content::UnitStruct(n) => serializer.serialize_unit_struct(n),
                Content::UnitVariant(n, i, v) => serializer.serialize_unit_variant(n, i, v),
                Content::NewtypeStruct(n, ref c) => serializer.serialize_newtype_struct(n, &**c),
                Content::NewtypeVariant(n, i, v, ref c) => {
                    serializer.serialize_newtype_variant(n, i, v, &**c)
                }
                Content::Seq(ref elements) => elements.serialize(serializer),
                Content::Tuple(ref elements) => {
                    use serde::ser::SerializeTuple;
                    let mut tuple = tri!(serializer.serialize_tuple(elements.len()));
                    for e in elements {
                        tri!(tuple.serialize_element(e));
                    }
                    tuple.end()
                }
                Content::TupleStruct(n, ref fields) => {
                    use serde::ser::SerializeTupleStruct;
                    let mut ts = tri!(serializer.serialize_tuple_struct(n, fields.len()));
                    for f in fields {
                        tri!(ts.serialize_field(f));
                    }
                    ts.end()
                }
                Content::TupleVariant(n, i, v, ref fields) => {
                    use serde::ser::SerializeTupleVariant;
                    let mut tv = tri!(serializer.serialize_tuple_variant(n, i, v, fields.len()));
                    for f in fields {
                        tri!(tv.serialize_field(f));
                    }
                    tv.end()
                }
                Content::Map(ref entries) => {
                    use serde::ser::SerializeMap;
                    let mut map = tri!(serializer.serialize_map(Some(entries.len())));
                    for (k, v) in entries {
                        tri!(map.serialize_entry(k, v));
                    }
                    map.end()
                }
                Content::Struct(n, ref fields) => {
                    use serde::ser::SerializeStruct;
                    let mut s = tri!(serializer.serialize_struct(n, fields.len()));
                    for &(k, ref v) in fields {
                        tri!(s.serialize_field(k, v));
                    }
                    s.end()
                }
                Content::StructVariant(n, i, v, ref fields) => {
                    use serde::ser::SerializeStructVariant;
                    let mut sv = tri!(serializer.serialize_struct_variant(n, i, v, fields.len()));
                    for &(k, ref v) in fields {
                        tri!(sv.serialize_field(k, v));
                    }
                    sv.end()
                }
            }
        }
    }

    pub struct ContentSerializer<E> {
        error: PhantomData<E>,
    }

    impl<E> ContentSerializer<E> {
        pub fn new() -> Self {
            ContentSerializer { error: PhantomData }
        }
    }

    impl<E> Serializer for ContentSerializer<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        type SerializeSeq = SerializeSeq<E>;
        type SerializeTuple = SerializeTuple<E>;
        type SerializeTupleStruct = SerializeTupleStruct<E>;
        type SerializeTupleVariant = SerializeTupleVariant<E>;
        type SerializeMap = SerializeMap<E>;
        type SerializeStruct = SerializeStruct<E>;
        type SerializeStructVariant = SerializeStructVariant<E>;

        fn serialize_bool(self, v: bool) -> Result<Content, E> {
            Ok(Content::Bool(v))
        }

        fn serialize_i8(self, v: i8) -> Result<Content, E> {
            Ok(Content::I8(v))
        }

        fn serialize_i16(self, v: i16) -> Result<Content, E> {
            Ok(Content::I16(v))
        }

        fn serialize_i32(self, v: i32) -> Result<Content, E> {
            Ok(Content::I32(v))
        }

        fn serialize_i64(self, v: i64) -> Result<Content, E> {
            Ok(Content::I64(v))
        }

        fn serialize_u8(self, v: u8) -> Result<Content, E> {
            Ok(Content::U8(v))
        }

        fn serialize_u16(self, v: u16) -> Result<Content, E> {
            Ok(Content::U16(v))
        }

        fn serialize_u32(self, v: u32) -> Result<Content, E> {
            Ok(Content::U32(v))
        }

        fn serialize_u64(self, v: u64) -> Result<Content, E> {
            Ok(Content::U64(v))
        }

        fn serialize_f32(self, v: f32) -> Result<Content, E> {
            Ok(Content::F32(v))
        }

        fn serialize_f64(self, v: f64) -> Result<Content, E> {
            Ok(Content::F64(v))
        }

        fn serialize_char(self, v: char) -> Result<Content, E> {
            Ok(Content::Char(v))
        }

        fn serialize_str(self, value: &str) -> Result<Content, E> {
            Ok(Content::String(value.to_owned()))
        }

        fn serialize_bytes(self, value: &[u8]) -> Result<Content, E> {
            Ok(Content::Bytes(value.to_owned()))
        }

        fn serialize_none(self) -> Result<Content, E> {
            Ok(Content::None)
        }

        fn serialize_some<T>(self, value: &T) -> Result<Content, E>
        where
            T: ?Sized + Serialize,
        {
            Ok(Content::Some(Box::new(tri!(value.serialize(self)))))
        }

        fn serialize_unit(self) -> Result<Content, E> {
            Ok(Content::Unit)
        }

        fn serialize_unit_struct(self, name: &'static str) -> Result<Content, E> {
            Ok(Content::UnitStruct(name))
        }

        fn serialize_unit_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
        ) -> Result<Content, E> {
            Ok(Content::UnitVariant(name, variant_index, variant))
        }

        fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Content, E>
        where
            T: ?Sized + Serialize,
        {
            Ok(Content::NewtypeStruct(
                name,
                Box::new(tri!(value.serialize(self))),
            ))
        }

        fn serialize_newtype_variant<T>(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            value: &T,
        ) -> Result<Content, E>
        where
            T: ?Sized + Serialize,
        {
            Ok(Content::NewtypeVariant(
                name,
                variant_index,
                variant,
                Box::new(tri!(value.serialize(self))),
            ))
        }

        fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, E> {
            Ok(SerializeSeq {
                elements: Vec::with_capacity(len.unwrap_or(0)),
                error: PhantomData,
            })
        }

        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, E> {
            Ok(SerializeTuple {
                elements: Vec::with_capacity(len),
                error: PhantomData,
            })
        }

        fn serialize_tuple_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleStruct, E> {
            Ok(SerializeTupleStruct {
                name,
                fields: Vec::with_capacity(len),
                error: PhantomData,
            })
        }

        fn serialize_tuple_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeTupleVariant, E> {
            Ok(SerializeTupleVariant {
                name,
                variant_index,
                variant,
                fields: Vec::with_capacity(len),
                error: PhantomData,
            })
        }

        fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, E> {
            Ok(SerializeMap {
                entries: Vec::with_capacity(len.unwrap_or(0)),
                key: None,
                error: PhantomData,
            })
        }

        fn serialize_struct(
            self,
            name: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStruct, E> {
            Ok(SerializeStruct {
                name,
                fields: Vec::with_capacity(len),
                error: PhantomData,
            })
        }

        fn serialize_struct_variant(
            self,
            name: &'static str,
            variant_index: u32,
            variant: &'static str,
            len: usize,
        ) -> Result<Self::SerializeStructVariant, E> {
            Ok(SerializeStructVariant {
                name,
                variant_index,
                variant,
                fields: Vec::with_capacity(len),
                error: PhantomData,
            })
        }
    }

    pub struct SerializeSeq<E> {
        elements: Vec<Content>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeSeq for SerializeSeq<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.elements.push(value);
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::Seq(self.elements))
        }
    }

    pub struct SerializeTuple<E> {
        elements: Vec<Content>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeTuple for SerializeTuple<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_element<T>(&mut self, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.elements.push(value);
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::Tuple(self.elements))
        }
    }

    pub struct SerializeTupleStruct<E> {
        name: &'static str,
        fields: Vec<Content>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeTupleStruct for SerializeTupleStruct<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_field<T>(&mut self, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.fields.push(value);
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::TupleStruct(self.name, self.fields))
        }
    }

    pub struct SerializeTupleVariant<E> {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<Content>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeTupleVariant for SerializeTupleVariant<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_field<T>(&mut self, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.fields.push(value);
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::TupleVariant(
                self.name,
                self.variant_index,
                self.variant,
                self.fields,
            ))
        }
    }

    pub struct SerializeMap<E> {
        entries: Vec<(Content, Content)>,
        key: Option<Content>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeMap for SerializeMap<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_key<T>(&mut self, key: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let key = tri!(key.serialize(ContentSerializer::<E>::new()));
            self.key = Some(key);
            Ok(())
        }

        fn serialize_value<T>(&mut self, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let key = self
                .key
                .take()
                .expect("serialize_value called before serialize_key");
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.entries.push((key, value));
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::Map(self.entries))
        }

        fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), E>
        where
            K: ?Sized + Serialize,
            V: ?Sized + Serialize,
        {
            let key = tri!(key.serialize(ContentSerializer::<E>::new()));
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.entries.push((key, value));
            Ok(())
        }
    }

    pub struct SerializeStruct<E> {
        name: &'static str,
        fields: Vec<(&'static str, Content)>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeStruct for SerializeStruct<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.fields.push((key, value));
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::Struct(self.name, self.fields))
        }
    }

    pub struct SerializeStructVariant<E> {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<(&'static str, Content)>,
        error: PhantomData<E>,
    }

    impl<E> ser::SerializeStructVariant for SerializeStructVariant<E>
    where
        E: ser::Error,
    {
        type Ok = Content;
        type Error = E;

        fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), E>
        where
            T: ?Sized + Serialize,
        {
            let value = tri!(value.serialize(ContentSerializer::<E>::new()));
            self.fields.push((key, value));
            Ok(())
        }

        fn end(self) -> Result<Content, E> {
            Ok(Content::StructVariant(
                self.name,
                self.variant_index,
                self.variant,
                self.fields,
            ))
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapSerializer<'a, M: 'a>(pub &'a mut M);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> FlatMapSerializer<'a, M>
where
    M: SerializeMap + 'a,
{
    fn bad_type(what: Unsupported) -> M::Error {
        ser::Error::custom(format_args!(
            "can only flatten structs and maps (got {})",
            what
        ))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> Serializer for FlatMapSerializer<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    type SerializeSeq = Impossible<Self::Ok, M::Error>;
    type SerializeTuple = Impossible<Self::Ok, M::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, M::Error>;
    type SerializeMap = FlatMapSerializeMap<'a, M>;
    type SerializeStruct = FlatMapSerializeStruct<'a, M>;
    type SerializeTupleVariant = FlatMapSerializeTupleVariantAsMapValue<'a, M>;
    type SerializeStructVariant = FlatMapSerializeStructVariantAsMapValue<'a, M>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Boolean))
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Integer))
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Float))
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Float))
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Char))
    }

    fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::String))
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::ByteArray))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::UnitStruct))
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Self::bad_type(Unsupported::Enum))
    }

    fn serialize_newtype_struct<T>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        tri!(self.0.serialize_key(variant));
        self.0.serialize_value(value)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Self::bad_type(Unsupported::Sequence))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Self::bad_type(Unsupported::Tuple))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Self::bad_type(Unsupported::TupleStruct))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        tri!(self.0.serialize_key(variant));
        Ok(FlatMapSerializeTupleVariantAsMapValue::new(self.0))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(FlatMapSerializeMap(self.0))
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(FlatMapSerializeStruct(self.0))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        inner_variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        tri!(self.0.serialize_key(inner_variant));
        Ok(FlatMapSerializeStructVariantAsMapValue::new(
            self.0,
            inner_variant,
        ))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapSerializeMap<'a, M: 'a>(&'a mut M);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> ser::SerializeMap for FlatMapSerializeMap<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_value(value)
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.0.serialize_entry(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapSerializeStruct<'a, M: 'a>(&'a mut M);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> ser::SerializeStruct for FlatMapSerializeStruct<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_entry(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapSerializeTupleVariantAsMapValue<'a, M: 'a> {
    map: &'a mut M,
    fields: Vec<Content>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> FlatMapSerializeTupleVariantAsMapValue<'a, M>
where
    M: SerializeMap + 'a,
{
    fn new(map: &'a mut M) -> Self {
        FlatMapSerializeTupleVariantAsMapValue {
            map,
            fields: Vec::new(),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> ser::SerializeTupleVariant for FlatMapSerializeTupleVariantAsMapValue<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = tri!(value.serialize(ContentSerializer::<M::Error>::new()));
        self.fields.push(value);
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        tri!(self.map.serialize_value(&Content::Seq(self.fields)));
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FlatMapSerializeStructVariantAsMapValue<'a, M: 'a> {
    map: &'a mut M,
    name: &'static str,
    fields: Vec<(&'static str, Content)>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> FlatMapSerializeStructVariantAsMapValue<'a, M>
where
    M: SerializeMap + 'a,
{
    fn new(map: &'a mut M, name: &'static str) -> FlatMapSerializeStructVariantAsMapValue<'a, M> {
        FlatMapSerializeStructVariantAsMapValue {
            map,
            name,
            fields: Vec::new(),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'a, M> ser::SerializeStructVariant for FlatMapSerializeStructVariantAsMapValue<'a, M>
where
    M: SerializeMap + 'a,
{
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = tri!(value.serialize(ContentSerializer::<M::Error>::new()));
        self.fields.push((key, value));
        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        tri!(self
            .map
            .serialize_value(&Content::Struct(self.name, self.fields)));
        Ok(())
    }
}

pub struct AdjacentlyTaggedEnumVariant {
    pub enum_name: &'static str,
    pub variant_index: u32,
    pub variant_name: &'static str,
}

impl Serialize for AdjacentlyTaggedEnumVariant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_unit_variant(self.enum_name, self.variant_index, self.variant_name)
    }
}

// Error when Serialize for a non_exhaustive remote enum encounters a variant
// that is not recognized.
pub struct CannotSerializeVariant<T>(pub T);

impl<T> Display for CannotSerializeVariant<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "enum variant cannot be serialized: {:?}", self.0)
    }
}
