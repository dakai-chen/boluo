use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserializer};

macro_rules! parse_single_value {
    ($trait_fn:ident, $visit_fn:ident, $ty:literal) => {
        fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            if self.path_params.len() != 1 {
                return Err(PathDeserializationError::WrongNumberOfParameters {
                    got: self.path_params.len(),
                    expected: 1,
                });
            }

            let value = self.path_params[0].1.as_str();
            let value = percent_encoding::percent_decode_str(value)
                .decode_utf8()
                .map_err(|_| PathDeserializationError::InvalidUtf8 {
                    value: value.to_owned(),
                })?;

            let value = value
                .parse()
                .map_err(|_| PathDeserializationError::ParseError {
                    value: value.into_owned(),
                    expected_type: $ty,
                })?;

            visitor.$visit_fn(value)
        }
    };
}

macro_rules! parse_key {
    ($trait_fn:ident) => {
        fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            visitor.visit_borrowed_str(self.key.as_str())
        }
    };
}

macro_rules! parse_value {
    ($trait_fn:ident, $visit_fn:ident, $ty:literal) => {
        fn $trait_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            let value = percent_encoding::percent_decode_str(self.value)
                .decode_utf8()
                .map_err(|_| PathDeserializationError::InvalidUtf8 {
                    value: self.value.to_owned(),
                })?;

            let v = value.parse().map_err(|_| match self.key {
                KeyOrIdx::Key(key) => PathDeserializationError::ParseErrorAtKey {
                    key: key.to_string(),
                    value: value.into_owned(),
                    expected_type: $ty,
                },
                KeyOrIdx::Idx(index) => PathDeserializationError::ParseErrorAtIndex {
                    index,
                    value: value.into_owned(),
                    expected_type: $ty,
                },
            })?;

            visitor.$visit_fn(v)
        }
    };
}

pub(super) struct PathDeserializer<'de> {
    path_params: &'de [(String, String)],
}

impl<'de> PathDeserializer<'de> {
    #[inline]
    pub(super) fn new(path_params: &'de [(String, String)]) -> Self {
        PathDeserializer { path_params }
    }
}

impl<'de> Deserializer<'de> for PathDeserializer<'de> {
    type Error = PathDeserializationError;

    fn deserialize_option<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    parse_single_value!(deserialize_bool, visit_bool, "bool");
    parse_single_value!(deserialize_i8, visit_i8, "i8");
    parse_single_value!(deserialize_i16, visit_i16, "i16");
    parse_single_value!(deserialize_i32, visit_i32, "i32");
    parse_single_value!(deserialize_i64, visit_i64, "i64");
    parse_single_value!(deserialize_i128, visit_i128, "i128");
    parse_single_value!(deserialize_u8, visit_u8, "u8");
    parse_single_value!(deserialize_u16, visit_u16, "u16");
    parse_single_value!(deserialize_u32, visit_u32, "u32");
    parse_single_value!(deserialize_u64, visit_u64, "u64");
    parse_single_value!(deserialize_u128, visit_u128, "u128");
    parse_single_value!(deserialize_f32, visit_f32, "f32");
    parse_single_value!(deserialize_f64, visit_f64, "f64");
    parse_single_value!(deserialize_str, visit_string, "String");
    parse_single_value!(deserialize_string, visit_string, "String");
    parse_single_value!(deserialize_bytes, visit_string, "String");
    parse_single_value!(deserialize_byte_buf, visit_string, "String");
    parse_single_value!(deserialize_char, visit_char, "char");

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer {
            params: self.path_params,
            idx: 0,
        })
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.path_params.len() != 0 {
            return Err(PathDeserializationError::WrongNumberOfParameters {
                got: self.path_params.len(),
                expected: 0,
            });
        }
        visitor.visit_unit()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.path_params.len() != len {
            return Err(PathDeserializationError::WrongNumberOfParameters {
                got: self.path_params.len(),
                expected: len,
            });
        }
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer {
            params: self.path_params,
            next: None,
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.path_params.len() != 1 {
            return Err(PathDeserializationError::WrongNumberOfParameters {
                got: self.path_params.len(),
                expected: 1,
            });
        }
        visitor.visit_enum(EnumDeserializer {
            value: &self.path_params[0].1,
        })
    }
}

struct SeqDeserializer<'de> {
    params: &'de [(String, String)],
    idx: usize,
}

impl<'de> SeqAccess<'de> for SeqDeserializer<'de> {
    type Error = PathDeserializationError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.params.split_first() {
            Some(((_, value), tail)) => {
                self.params = tail;
                let idx = self.idx;
                self.idx += 1;
                Ok(Some(seed.deserialize(ValueDeserializer {
                    key: KeyOrIdx::Idx(idx),
                    value,
                })?))
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.params.len())
    }
}

struct MapDeserializer<'de> {
    params: &'de [(String, String)],
    next: Option<(KeyOrIdx<'de>, &'de String)>,
}

impl<'de> MapAccess<'de> for MapDeserializer<'de> {
    type Error = PathDeserializationError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.params.split_first() {
            Some(((key, value), tail)) => {
                self.params = tail;
                self.next = Some((KeyOrIdx::Key(key), value));
                seed.deserialize(KeyDeserializer { key }).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.next.take() {
            seed.deserialize(ValueDeserializer { key, value })
        } else {
            panic!("calling next_value_seed before next_key_seed is incorrect")
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.params.len())
    }
}

struct KeyDeserializer<'de> {
    key: &'de String,
}

impl<'de> Deserializer<'de> for KeyDeserializer<'de> {
    type Error = PathDeserializationError;

    parse_key!(deserialize_identifier);
    parse_key!(deserialize_str);
    parse_key!(deserialize_string);

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedKeyType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char bytes
        byte_buf option unit unit_struct seq tuple
        tuple_struct map newtype_struct struct enum ignored_any
    }
}

#[derive(Debug)]
struct ValueDeserializer<'de> {
    key: KeyOrIdx<'de>,
    value: &'de String,
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = PathDeserializationError;

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: std::any::type_name::<V::Value>(),
        })
    }

    parse_value!(deserialize_bool, visit_bool, "bool");
    parse_value!(deserialize_i8, visit_i8, "i8");
    parse_value!(deserialize_i16, visit_i16, "i16");
    parse_value!(deserialize_i32, visit_i32, "i32");
    parse_value!(deserialize_i64, visit_i64, "i64");
    parse_value!(deserialize_i128, visit_i128, "i128");
    parse_value!(deserialize_u8, visit_u8, "u8");
    parse_value!(deserialize_u16, visit_u16, "u16");
    parse_value!(deserialize_u32, visit_u32, "u32");
    parse_value!(deserialize_u64, visit_u64, "u64");
    parse_value!(deserialize_u128, visit_u128, "u128");
    parse_value!(deserialize_f32, visit_f32, "f32");
    parse_value!(deserialize_f64, visit_f64, "f64");
    parse_value!(deserialize_str, visit_string, "String");
    parse_value!(deserialize_string, visit_string, "String");
    parse_value!(deserialize_bytes, visit_string, "String");
    parse_value!(deserialize_byte_buf, visit_string, "String");
    parse_value!(deserialize_char, visit_char, "char");

    fn deserialize_any<V>(self, v: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(v)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(EnumDeserializer { value: self.value })
    }
}

struct EnumDeserializer<'de> {
    value: &'de String,
}

impl<'de> EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = PathDeserializationError;
    type Variant = UnitVariant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((
            seed.deserialize(KeyDeserializer { key: self.value })?,
            UnitVariant,
        ))
    }
}

struct UnitVariant;

impl<'de> VariantAccess<'de> for UnitVariant {
    type Error = PathDeserializationError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: "newtype enum variant",
        })
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: "tuple enum variant",
        })
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(PathDeserializationError::UnsupportedValueType {
            name: "struct enum variant",
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum KeyOrIdx<'de> {
    Key(&'de String),
    Idx(usize),
}

#[derive(Debug)]
pub(super) enum PathDeserializationError {
    /// 参数数量不正确
    WrongNumberOfParameters { got: usize, expected: usize },

    /// 尝试反序列化为不受支持的键类型
    UnsupportedKeyType { name: &'static str },

    /// 尝试反序列化为不受支持的值类型
    UnsupportedValueType { name: &'static str },

    /// 无法将特定键处的值解析为所需类型
    ParseErrorAtKey {
        key: String,
        value: String,
        expected_type: &'static str,
    },

    /// 无法将特定索引处的值解析为所需的类型
    ParseErrorAtIndex {
        index: usize,
        value: String,
        expected_type: &'static str,
    },

    /// 无法将值解析为所需的类型
    ParseError {
        value: String,
        expected_type: &'static str,
    },

    /// 无法将值解析为有效的UTF-8字符串
    InvalidUtf8 { value: String },

    /// 捕获所有不适合任何其他变体的错误变体
    Message(String),
}

impl std::fmt::Display for PathDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathDeserializationError::WrongNumberOfParameters { got, expected } => {
                write!(
                    f,
                    "wrong number of path arguments. expected {expected} but got {got}"
                )
            }
            PathDeserializationError::UnsupportedKeyType { name } => {
                write!(f, "unsupported key type `{name}`")
            }
            PathDeserializationError::UnsupportedValueType { name } => {
                write!(f, "unsupported value type `{name}`")
            }
            PathDeserializationError::ParseErrorAtKey {
                key,
                value,
                expected_type,
            } => write!(
                f,
                "cannot parse `{key}` with value `{value:?}` to a `{expected_type}`"
            ),
            PathDeserializationError::ParseErrorAtIndex {
                index,
                value,
                expected_type,
            } => write!(
                f,
                "cannot parse value at index {index} with value `{value:?}` to a `{expected_type}`"
            ),
            PathDeserializationError::ParseError {
                value,
                expected_type,
            } => write!(f, "cannot parse `{value:?}` to a `{expected_type}`"),
            PathDeserializationError::InvalidUtf8 { value } => {
                write!(f, "cannot parse `{value:?}` to a UTF-8 string")
            }
            PathDeserializationError::Message(msg) => msg.fmt(f),
        }
    }
}

impl std::error::Error for PathDeserializationError {}

impl serde::de::Error for PathDeserializationError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        PathDeserializationError::Message(msg.to_string())
    }
}
