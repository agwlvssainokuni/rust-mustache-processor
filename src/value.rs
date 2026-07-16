// Copyright 2026 agwlvssainokuni
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Mustacheのレンダリングに使用する内部データ表現。

use std::fmt;

use serde::Serialize;
use serde::ser::{
    self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};

/// Mustacheのコンテキストとして扱う、フォーマット非依存の内部データ表現。
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// 値が存在しないことを表す。
    Null,
    /// 真偽値。
    Bool(bool),
    /// 整数値。
    Integer(i64),
    /// 浮動小数点数値。
    Float(f64),
    /// 文字列値。
    String(String),
    /// 配列値。
    Array(Vec<Value>),
    /// キー順序を保持するマップ値。
    Map(Map),
}

impl Value {
    /// 任意の`Serialize`実装型からValueへ変換する。
    pub fn from_serialize<T: Serialize + ?Sized>(value: &T) -> Result<Value, ValueError> {
        value.serialize(ValueSerializer)
    }

    /// セクション評価用の真偽判定（Mustache仕様のfalsy定義に従う。BR-2.1〜BR-2.4）。
    ///
    /// 空文字列・空Mapは（他の一部テンプレートエンジンと異なり）truthyとして扱う。
    /// Mustache仕様上のfalsy値は`false`・`null`・空配列のみである。
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Array(items) => !items.is_empty(),
            Value::Map(_) => true,
            Value::Integer(_) | Value::Float(_) | Value::String(_) => true,
        }
    }

    /// Mapからのキー参照（コンテキスト探索に使用）。Map以外は`None`を返す。
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(map) => map.get(key),
            _ => None,
        }
    }

    /// Arrayの場合の繰り返し評価用イテレータ。Array以外は`None`を返す。
    pub fn iter(&self) -> Option<impl Iterator<Item = &Value>> {
        match self {
            Value::Array(items) => Some(items.iter()),
            _ => None,
        }
    }
}

/// キーの挿入順序を保持するマップ型。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Map {
    entries: Vec<(String, Value)>,
}

impl Map {
    /// 空のMapを作成する。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// キーに値を関連付ける。既存のキーであれば値を上書きし、旧値を返す。
    pub fn insert(&mut self, key: impl Into<String>, value: Value) -> Option<Value> {
        let key = key.into();
        if let Some(entry) = self.entries.iter_mut().find(|(k, _)| *k == key) {
            Some(std::mem::replace(&mut entry.1, value))
        } else {
            self.entries.push((key, value));
            None
        }
    }

    /// キーに対応する値への参照を返す。
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// 挿入順序でキーと値のペアを走査するイテレータを返す。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// 要素数を返す。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 要素が空かどうかを返す。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// `Value::from_serialize`で発生し得るエラー。
#[derive(Debug, Clone, PartialEq)]
pub struct ValueError {
    message: String,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValueError {}

impl ser::Error for ValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ValueError {
            message: msg.to_string(),
        }
    }
}

struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = ValueError;
    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value, ValueError> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Value, ValueError> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i16(self, v: i16) -> Result<Value, ValueError> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i32(self, v: i32) -> Result<Value, ValueError> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_i64(self, v: i64) -> Result<Value, ValueError> {
        Ok(Value::Integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Value, ValueError> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u16(self, v: u16) -> Result<Value, ValueError> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u32(self, v: u32) -> Result<Value, ValueError> {
        Ok(Value::Integer(v as i64))
    }

    fn serialize_u64(self, v: u64) -> Result<Value, ValueError> {
        i64::try_from(v)
            .map(Value::Integer)
            .map_err(ser::Error::custom)
    }

    fn serialize_f32(self, v: f32) -> Result<Value, ValueError> {
        Ok(Value::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Value, ValueError> {
        Ok(Value::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Value, ValueError> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Value, ValueError> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Value, ValueError> {
        Ok(Value::Array(
            v.iter().map(|b| Value::Integer(*b as i64)).collect(),
        ))
    }

    fn serialize_none(self) -> Result<Value, ValueError> {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value, ValueError> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Value, ValueError> {
        Ok(Value::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value, ValueError> {
        Ok(Value::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value, ValueError> {
        Ok(Value::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value, ValueError> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value, ValueError> {
        let mut map = Map::new();
        map.insert(variant, Value::from_serialize(value)?);
        Ok(Value::Map(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer, ValueError> {
        Ok(SeqSerializer {
            items: Vec::with_capacity(len.unwrap_or(0)),
            variant: None,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer, ValueError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, ValueError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<SeqSerializer, ValueError> {
        Ok(SeqSerializer {
            items: Vec::with_capacity(len),
            variant: Some(variant),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<MapSerializer, ValueError> {
        Ok(MapSerializer {
            map: Map::new(),
            next_key: None,
            variant: None,
            _len_hint: len,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<MapSerializer, ValueError> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<MapSerializer, ValueError> {
        Ok(MapSerializer {
            map: Map::new(),
            next_key: None,
            variant: Some(variant),
            _len_hint: Some(len),
        })
    }
}

struct SeqSerializer {
    items: Vec<Value>,
    variant: Option<&'static str>,
}

fn finish_seq(items: Vec<Value>, variant: Option<&'static str>) -> Value {
    let array = Value::Array(items);
    match variant {
        Some(v) => {
            let mut outer = Map::new();
            outer.insert(v, array);
            Value::Map(outer)
        }
        None => array,
    }
}

impl SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), ValueError> {
        self.items.push(Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_seq(self.items, self.variant))
    }
}

impl SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), ValueError> {
        self.items.push(Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_seq(self.items, self.variant))
    }
}

impl SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), ValueError> {
        self.items.push(Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_seq(self.items, self.variant))
    }
}

impl SerializeTupleVariant for SeqSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), ValueError> {
        self.items.push(Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_seq(self.items, self.variant))
    }
}

struct MapSerializer {
    map: Map,
    next_key: Option<String>,
    variant: Option<&'static str>,
    _len_hint: Option<usize>,
}

fn finish_map(map: Map, variant: Option<&'static str>) -> Value {
    match variant {
        Some(v) => {
            let mut outer = Map::new();
            outer.insert(v, Value::Map(map));
            Value::Map(outer)
        }
        None => Value::Map(map),
    }
}

/// Mapのキーをシリアライズするための限定的なシリアライザ。
///
/// Mustacheのコンテキストキーは文字列であるため、プリミティブ型を文字列表現に変換する。
/// 配列・マップ等の複合型はキーにできないためエラーとする。
struct MapKeySerializer;

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = ValueError;
    type SerializeSeq = ser::Impossible<String, ValueError>;
    type SerializeTuple = ser::Impossible<String, ValueError>;
    type SerializeTupleStruct = ser::Impossible<String, ValueError>;
    type SerializeTupleVariant = ser::Impossible<String, ValueError>;
    type SerializeMap = ser::Impossible<String, ValueError>;
    type SerializeStruct = ser::Impossible<String, ValueError>;
    type SerializeStructVariant = ser::Impossible<String, ValueError>;

    fn serialize_bool(self, v: bool) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_i8(self, v: i8) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_i16(self, v: i16) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_i32(self, v: i32) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_i64(self, v: i64) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_u8(self, v: u8) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_u16(self, v: u16) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_u32(self, v: u32) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_u64(self, v: u64) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_f32(self, v: f32) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_f64(self, v: f64) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_char(self, v: char) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_str(self, v: &str) -> Result<String, ValueError> {
        Ok(v.to_string())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<String, ValueError> {
        Err(ser::Error::custom("map key must not be bytes"))
    }
    fn serialize_none(self) -> Result<String, ValueError> {
        Err(ser::Error::custom("map key must not be none"))
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<String, ValueError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<String, ValueError> {
        Err(ser::Error::custom("map key must not be unit"))
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<String, ValueError> {
        Ok(name.to_string())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String, ValueError> {
        Ok(variant.to_string())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<String, ValueError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, ValueError> {
        Err(ser::Error::custom("map key must not be a newtype variant"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, ValueError> {
        Err(ser::Error::custom("map key must not be a sequence"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, ValueError> {
        Err(ser::Error::custom("map key must not be a tuple"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, ValueError> {
        Err(ser::Error::custom("map key must not be a tuple struct"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, ValueError> {
        Err(ser::Error::custom("map key must not be a tuple variant"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, ValueError> {
        Err(ser::Error::custom("map key must not be a map"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, ValueError> {
        Err(ser::Error::custom("map key must not be a struct"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, ValueError> {
        Err(ser::Error::custom("map key must not be a struct variant"))
    }
}

impl SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), ValueError> {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), ValueError> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| ser::Error::custom("serialize_value called before serialize_key"))?;
        self.map.insert(key, Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_map(self.map, self.variant))
    }
}

impl SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), ValueError> {
        self.map.insert(key, Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_map(self.map, self.variant))
    }
}

impl SerializeStructVariant for MapSerializer {
    type Ok = Value;
    type Error = ValueError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), ValueError> {
        self.map.insert(key, Value::from_serialize(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, ValueError> {
        Ok(finish_map(self.map, self.variant))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn is_truthy_null_and_bool() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(Value::Bool(true).is_truthy());
    }

    #[test]
    fn is_truthy_array() {
        assert!(!Value::Array(vec![]).is_truthy());
        assert!(Value::Array(vec![Value::Null]).is_truthy());
    }

    #[test]
    fn is_truthy_empty_string_and_empty_map_are_truthy() {
        // BR-2.1〜BR-2.4: 公式spec準拠で空文字列・空Mapはtruthy
        assert!(Value::String(String::new()).is_truthy());
        assert!(Value::Map(Map::new()).is_truthy());
    }

    #[test]
    fn is_truthy_numbers_including_zero() {
        // 0も0.0も非0の数値と同様に真として扱う
        assert!(Value::Integer(0).is_truthy());
        assert!(Value::Float(0.0).is_truthy());
        assert!(Value::Integer(-1).is_truthy());
    }

    #[test]
    fn get_on_map_and_non_map() {
        let mut map = Map::new();
        map.insert("k", Value::Integer(1));
        let v = Value::Map(map);
        assert_eq!(v.get("k"), Some(&Value::Integer(1)));
        assert_eq!(v.get("missing"), None);
        assert_eq!(Value::Integer(1).get("k"), None);
    }

    #[test]
    fn iter_on_array_and_non_array() {
        let v = Value::Array(vec![Value::Integer(1), Value::Integer(2)]);
        let collected: Vec<&Value> = v.iter().unwrap().collect();
        assert_eq!(collected, vec![&Value::Integer(1), &Value::Integer(2)]);
        assert!(Value::Integer(1).iter().is_none());
    }

    #[test]
    fn map_preserves_insertion_order() {
        let mut map = Map::new();
        map.insert("z", Value::Integer(1));
        map.insert("a", Value::Integer(2));
        map.insert("m", Value::Integer(3));
        let keys: Vec<&str> = map.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn map_insert_overwrites_existing_key_without_reordering() {
        let mut map = Map::new();
        map.insert("a", Value::Integer(1));
        map.insert("b", Value::Integer(2));
        let old = map.insert("a", Value::Integer(99));
        assert_eq!(old, Some(Value::Integer(1)));
        let keys: Vec<&str> = map.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, vec!["a", "b"]);
        assert_eq!(map.get("a"), Some(&Value::Integer(99)));
    }

    #[derive(Serialize)]
    struct Person {
        name: String,
        age: u32,
        active: bool,
        nickname: Option<String>,
    }

    #[test]
    fn from_serialize_struct() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            active: true,
            nickname: None,
        };
        let value = Value::from_serialize(&person).unwrap();
        match value {
            Value::Map(map) => {
                assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
                assert_eq!(map.get("age"), Some(&Value::Integer(30)));
                assert_eq!(map.get("active"), Some(&Value::Bool(true)));
                assert_eq!(map.get("nickname"), Some(&Value::Null));
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }

    #[test]
    fn from_serialize_primitives() {
        assert_eq!(Value::from_serialize(&42i32).unwrap(), Value::Integer(42));
        assert_eq!(Value::from_serialize(&1.5f64).unwrap(), Value::Float(1.5));
        assert_eq!(
            Value::from_serialize(&"hi").unwrap(),
            Value::String("hi".to_string())
        );
        assert_eq!(Value::from_serialize(&true).unwrap(), Value::Bool(true));
    }

    #[test]
    fn from_serialize_vec() {
        let v = vec![1, 2, 3];
        let value = Value::from_serialize(&v).unwrap();
        assert_eq!(
            value,
            Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        );
    }

    #[test]
    fn from_serialize_map() {
        let mut m = std::collections::BTreeMap::new();
        m.insert("x", 1);
        m.insert("y", 2);
        let value = Value::from_serialize(&m).unwrap();
        match value {
            Value::Map(map) => {
                assert_eq!(map.get("x"), Some(&Value::Integer(1)));
                assert_eq!(map.get("y"), Some(&Value::Integer(2)));
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }

    #[test]
    fn from_serialize_option_some() {
        let v: Option<i32> = Some(5);
        assert_eq!(Value::from_serialize(&v).unwrap(), Value::Integer(5));
    }

    #[test]
    fn from_serialize_nested_struct() {
        #[derive(Serialize)]
        struct Outer {
            items: Vec<Person>,
        }
        let outer = Outer {
            items: vec![Person {
                name: "Bob".to_string(),
                age: 20,
                active: false,
                nickname: Some("B".to_string()),
            }],
        };
        let value = Value::from_serialize(&outer).unwrap();
        match value {
            Value::Map(map) => match map.get("items") {
                Some(Value::Array(items)) => {
                    assert_eq!(items.len(), 1);
                    match &items[0] {
                        Value::Map(inner) => {
                            assert_eq!(
                                inner.get("nickname"),
                                Some(&Value::String("B".to_string()))
                            );
                        }
                        other => panic!("expected Map, got {other:?}"),
                    }
                }
                other => panic!("expected Array, got {other:?}"),
            },
            other => panic!("expected Map, got {other:?}"),
        }
    }
}
