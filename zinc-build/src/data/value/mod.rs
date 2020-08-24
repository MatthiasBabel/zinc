//!
//! The Zinc VM template value.
//!

pub mod error;
pub mod scalar;

use std::collections::HashSet;

use bson::Bson;
use bson::Document as BsonDocument;
use num_bigint::BigInt;
use num_traits::Num;
use num_traits::Zero;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;

use crate::data::r#type::scalar::integer::Type as IntegerType;
use crate::data::r#type::scalar::Type as ScalarType;
use crate::data::r#type::Type as BuildType;

use self::error::context::IContext as IErrorContext;
use self::error::r#type::Type as ErrorType;
use self::error::Error;
use self::scalar::Value as ScalarValue;

///
/// The Zinc VM template value.
///
/// The representation of the witness and public data stored in JSON files.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Represented with `()` string.
    Unit,
    /// See the inner element description.
    Scalar(ScalarValue),
    /// Represented with JSON array.
    Array(Vec<Value>),
    /// Represented with JSON object.
    Structure(Vec<(String, Value)>),
    /// Represented with JSON object.
    Contract(Vec<(String, Value)>),
}

impl Value {
    ///
    /// Creates a value of `r#type`.
    ///
    pub fn new(r#type: BuildType) -> Self {
        match r#type {
            BuildType::Unit => Self::Unit,
            BuildType::Scalar(scalar_type) => match scalar_type {
                ScalarType::Boolean => Self::Scalar(ScalarValue::Boolean(false)),
                ScalarType::Integer(inner) => Self::Scalar(ScalarValue::Integer(0.into(), inner)),
                ScalarType::Field => Self::Scalar(ScalarValue::Field(0.into())),
            },
            BuildType::Enumeration => Self::Scalar(ScalarValue::Field(0.into())),

            BuildType::Array(r#type, size) => Self::Array(vec![Self::new(*r#type); size]),
            BuildType::Tuple(fields) => Self::Array(fields.into_iter().map(Self::new).collect()),
            BuildType::Structure(fields) => Self::Structure(
                fields
                    .into_iter()
                    .map(|(name, r#type)| (name, Self::new(r#type)))
                    .collect(),
            ),
            BuildType::Contract(fields) => Self::Contract(
                fields
                    .into_iter()
                    .map(|(name, r#type)| (name, Self::new(r#type)))
                    .collect(),
            ),
        }
    }

    ///
    /// Creates a value of `r#type` from the JSON `value`.
    ///
    pub fn try_from_typed_json(value: JsonValue, r#type: BuildType) -> Result<Self, Error> {
        match r#type {
            BuildType::Unit => Self::unit_from_json(value),
            BuildType::Scalar(inner) => Self::scalar_from_json(value, inner),
            BuildType::Enumeration => Self::field_from_json(value),

            BuildType::Array(inner, size) => Self::array_from_json(value, *inner, size),
            BuildType::Tuple(inner) => Self::tuple_from_json(value, inner),
            BuildType::Structure(fields) => Self::structure_from_json(value, fields),
            BuildType::Contract(fields) => Self::contract_from_json(value, fields),
        }
    }

    ///
    /// Creates a value from a flat array `flat_values` and data `r#type`.
    ///
    pub fn from_flat_values(r#type: BuildType, flat_values: &[BigInt]) -> Self {
        match r#type {
            BuildType::Unit => Self::Unit,
            BuildType::Scalar(r#type) => match r#type {
                ScalarType::Boolean => flat_values
                    .first()
                    .cloned()
                    .map(|value| value != BigInt::zero())
                    .map(ScalarValue::Boolean)
                    .map(Self::Scalar),
                ScalarType::Integer(r#type) => flat_values
                    .first()
                    .cloned()
                    .map(|value| ScalarValue::Integer(value, r#type))
                    .map(Self::Scalar),
                ScalarType::Field => flat_values
                    .first()
                    .cloned()
                    .map(ScalarValue::Field)
                    .map(Self::Scalar),
            }
            .expect(zinc_const::panic::VALUE_ALWAYS_EXISTS),
            BuildType::Enumeration => flat_values
                .first()
                .cloned()
                .map(ScalarValue::Field)
                .map(Self::Scalar)
                .expect(zinc_const::panic::VALUE_ALWAYS_EXISTS),
            BuildType::Array(r#type, size) => {
                let mut offset = 0;
                let mut result = Vec::with_capacity(size);
                for _ in 0..size {
                    let slice = &flat_values[offset..];
                    offset += r#type.size();
                    result.push(Self::from_flat_values(*r#type.clone(), slice));
                }
                Self::Array(result)
            }
            BuildType::Tuple(types) => {
                let mut offset = 0;
                let mut result = Vec::with_capacity(types.len());
                for r#type in types.into_iter() {
                    let slice = &flat_values[offset..];
                    offset += r#type.size();
                    result.push(Self::from_flat_values(r#type, slice));
                }
                Self::Array(result)
            }
            BuildType::Structure(fields) => {
                let mut offset = 0;
                let mut result = Vec::with_capacity(fields.len());
                for (name, r#type) in fields.into_iter() {
                    let slice = &flat_values[offset..];
                    offset += r#type.size();
                    result.push((name, Self::from_flat_values(r#type, slice)));
                }
                Self::Structure(result)
            }
            BuildType::Contract(fields) => {
                let mut offset = 0;
                let mut result = Vec::with_capacity(fields.len());
                for (name, r#type) in fields.into_iter() {
                    let slice = &flat_values[offset..];
                    offset += r#type.size();
                    result.push((name, Self::from_flat_values(r#type, slice)));
                }
                Self::Contract(result)
            }
        }
    }

    ///
    /// Flattens the value into an array of `BigInt`s.
    ///
    /// Is used to write the input to the VM data stack.
    ///
    pub fn into_flat_values(self) -> Vec<BigInt> {
        match self {
            Self::Unit => vec![],
            Self::Scalar(value) => vec![value.to_bigint()],
            Self::Array(values) => values
                .into_iter()
                .map(Self::into_flat_values)
                .flatten()
                .collect(),
            Self::Structure(fields) => fields
                .into_iter()
                .map(|(_name, value)| Self::into_flat_values(value))
                .flatten()
                .collect(),
            Self::Contract(fields) => fields
                .into_iter()
                .map(|(_name, value)| Self::into_flat_values(value))
                .flatten()
                .collect(),
        }
    }

    ///
    /// Converts the value to a JSON value.
    ///
    /// Is used to write the value to a witness or public data JSON file.
    ///
    pub fn into_json(self) -> JsonValue {
        match self {
            Self::Unit => JsonValue::Null,
            Self::Scalar(scalar) => match scalar {
                ScalarValue::Field(value) => {
                    if value <= BigInt::from(std::u64::MAX) {
                        JsonValue::String(value.to_str_radix(zinc_const::base::DECIMAL as u32))
                    } else {
                        JsonValue::String(
                            String::from("0x")
                                + value
                                    .to_str_radix(zinc_const::base::HEXADECIMAL as u32)
                                    .as_str(),
                        )
                    }
                }
                ScalarValue::Integer(value, r#type) => {
                    if value <= BigInt::from(std::u64::MAX) || r#type.is_signed {
                        JsonValue::String(value.to_str_radix(zinc_const::base::DECIMAL as u32))
                    } else {
                        JsonValue::String(
                            String::from("0x")
                                + value
                                    .to_str_radix(zinc_const::base::HEXADECIMAL as u32)
                                    .as_str(),
                        )
                    }
                }
                ScalarValue::Boolean(value) => JsonValue::Bool(value),
            },
            Self::Array(values) => {
                JsonValue::Array(values.into_iter().map(Self::into_json).collect())
            }
            Self::Structure(fields) => {
                let mut object = JsonMap::<String, JsonValue>::with_capacity(fields.len());
                for (name, value) in fields.into_iter() {
                    object.insert(name, Self::into_json(value));
                }
                JsonValue::Object(object)
            }
            Self::Contract(fields) => {
                let mut object = JsonMap::<String, JsonValue>::with_capacity(fields.len());
                for (name, value) in fields.into_iter() {
                    object.insert(name, Self::into_json(value));
                }
                JsonValue::Object(object)
            }
        }
    }

    ///
    /// Converts the value to a BSON value.
    ///
    /// Is used to write the value to the MongoDB.
    ///
    pub fn into_bson(self) -> Bson {
        match self {
            Self::Unit => Bson::Null,
            Self::Scalar(scalar) => match scalar {
                ScalarValue::Field(value) => {
                    if value <= BigInt::from(std::u64::MAX) {
                        Bson::String(value.to_str_radix(zinc_const::base::DECIMAL as u32))
                    } else {
                        Bson::String(
                            String::from("0x")
                                + value
                                    .to_str_radix(zinc_const::base::HEXADECIMAL as u32)
                                    .as_str(),
                        )
                    }
                }
                ScalarValue::Integer(value, r#type) => {
                    if value <= BigInt::from(std::u64::MAX) || r#type.is_signed {
                        Bson::String(value.to_str_radix(zinc_const::base::DECIMAL as u32))
                    } else {
                        Bson::String(
                            String::from("0x")
                                + value
                                    .to_str_radix(zinc_const::base::HEXADECIMAL as u32)
                                    .as_str(),
                        )
                    }
                }
                ScalarValue::Boolean(value) => Bson::Boolean(value),
            },
            Self::Array(values) => Bson::Array(values.into_iter().map(Self::into_bson).collect()),
            Self::Structure(fields) => {
                let mut object = BsonDocument::new();
                for (name, value) in fields.into_iter() {
                    object.insert(name, Self::into_bson(value));
                }
                Bson::Document(object)
            }
            Self::Contract(fields) => {
                let mut object = BsonDocument::new();
                for (name, value) in fields.into_iter() {
                    object.insert(name, Self::into_bson(value));
                }
                Bson::Document(object)
            }
        }
    }

    ///
    /// Creates a unit value from the JSON `value`.
    ///
    fn unit_from_json(value: JsonValue) -> Result<Self, Error> {
        if value.is_null() {
            return Ok(Self::Unit);
        }

        Err(ErrorType::TypeError {
            expected: "()".into(),
            found: value.to_string(),
        }
        .into())
    }

    ///
    /// Creates a boolean value from the JSON `value`.
    ///
    fn boolean_from_json(value: JsonValue) -> Result<Self, Error> {
        let value_bool = value.as_bool().ok_or_else(|| ErrorType::TypeError {
            expected: "boolean (true or false)".into(),
            found: value.to_string(),
        })?;

        Ok(Self::Scalar(ScalarValue::Boolean(value_bool)))
    }

    ///
    /// Creates an integer value from the JSON `value`.
    ///
    fn integer_from_json(value: JsonValue, r#type: IntegerType) -> Result<Self, Error> {
        let value_string = value.as_str().ok_or_else(|| ErrorType::TypeError {
            expected: "integer (number string)".into(),
            found: value.to_string(),
        })?;

        let bigint_result = if value_string.starts_with("0b") {
            BigInt::from_str_radix(&value_string[2..], zinc_const::base::BINARY as u32)
        } else if value_string.starts_with("0o") {
            BigInt::from_str_radix(&value_string[2..], zinc_const::base::OCTAL as u32)
        } else if value_string.starts_with("0x") {
            BigInt::from_str_radix(&value_string[2..], zinc_const::base::HEXADECIMAL as u32)
        } else {
            BigInt::from_str_radix(value_string, zinc_const::base::DECIMAL as u32)
        };

        let bigint =
            bigint_result.map_err(|_| ErrorType::InvalidNumberFormat(value_string.into()))?;

        // TODO: overflow check

        Ok(Self::Scalar(ScalarValue::Integer(bigint, r#type)))
    }

    ///
    /// Creates a field value from the JSON `value`.
    ///
    fn field_from_json(value: JsonValue) -> Result<Self, Error> {
        let value_string = value.as_str().ok_or_else(|| ErrorType::TypeError {
            expected: "field (number string)".into(),
            found: value.to_string(),
        })?;

        let bigint_result = if value_string.starts_with("0b") {
            BigInt::from_str_radix(&value_string[2..], zinc_const::base::BINARY as u32)
        } else if value_string.starts_with("0o") {
            BigInt::from_str_radix(&value_string[2..], zinc_const::base::OCTAL as u32)
        } else if value_string.starts_with("0x") {
            BigInt::from_str_radix(&value_string[2..], zinc_const::base::HEXADECIMAL as u32)
        } else {
            BigInt::from_str_radix(value_string, zinc_const::base::DECIMAL as u32)
        };

        let bigint =
            bigint_result.map_err(|_| ErrorType::InvalidNumberFormat(value_string.into()))?;

        // TODO: overflow check

        Ok(Self::Scalar(ScalarValue::Field(bigint)))
    }

    ///
    /// Creates a scalar value from the JSON `value`.
    ///
    fn scalar_from_json(value: JsonValue, scalar_type: ScalarType) -> Result<Self, Error> {
        match scalar_type {
            ScalarType::Boolean => Self::boolean_from_json(value),
            ScalarType::Integer(inner) => Self::integer_from_json(value, inner),
            ScalarType::Field => Self::field_from_json(value),
        }
    }

    ///
    /// Creates an array value from the JSON `value`.
    ///
    fn array_from_json(value: JsonValue, r#type: BuildType, size: usize) -> Result<Self, Error> {
        let array = value
            .as_array()
            .cloned()
            .ok_or_else(|| ErrorType::type_error("array".into(), value))?;

        if array.len() != size {
            return Err(ErrorType::UnexpectedSize {
                expected: size,
                found: array.len(),
            }
            .into());
        }

        let mut values = Vec::with_capacity(size);
        for (index, value) in array.into_iter().enumerate() {
            let typed_value = Self::try_from_typed_json(value, r#type.clone()).push_array(index)?;

            values.push(typed_value);
        }

        Ok(Self::Array(values))
    }

    ///
    /// Creates a tuple value from the JSON `value`.
    ///
    fn tuple_from_json(value: JsonValue, types: Vec<BuildType>) -> Result<Self, Error> {
        let array = value
            .as_array()
            .cloned()
            .ok_or_else(|| ErrorType::type_error("tuple (json array)".into(), value))?;

        if array.len() != types.len() {
            return Err(ErrorType::UnexpectedSize {
                expected: types.len(),
                found: array.len(),
            }
            .into());
        }

        let mut values = Vec::with_capacity(types.len());
        for (index, (value, r#type)) in array.into_iter().zip(types).enumerate() {
            let typed_value = Self::try_from_typed_json(value, r#type).push_array(index)?;
            values.push(typed_value);
        }

        Ok(Self::Array(values))
    }

    ///
    /// Creates a structure value from the JSON `value`.
    ///
    fn structure_from_json(
        value: JsonValue,
        field_types: Vec<(String, BuildType)>,
    ) -> Result<Self, Error> {
        let mut object = value
            .as_object()
            .cloned()
            .ok_or_else(|| ErrorType::type_error("structure".into(), value))?;

        let mut used_fields = HashSet::with_capacity(field_types.len());
        let mut field_values = Vec::with_capacity(field_types.len());
        for (name, r#type) in field_types.into_iter() {
            used_fields.insert(name.clone());

            let json_value = object
                .remove(name.as_str())
                .ok_or_else(|| ErrorType::MissingField(name.clone()))?;

            let value =
                Self::try_from_typed_json(json_value, r#type).push_structure(name.as_str())?;

            field_values.push((name, value));
        }

        for field in object.keys() {
            if !used_fields.contains(field.as_str()) {
                return Err(ErrorType::UnexpectedField(field.clone()).into());
            }
        }

        Ok(Self::Structure(field_values))
    }

    ///
    /// Creates a contract value from the JSON `value`.
    ///
    fn contract_from_json(
        value: JsonValue,
        field_types: Vec<(String, BuildType)>,
    ) -> Result<Self, Error> {
        let mut object = value
            .as_object()
            .cloned()
            .ok_or_else(|| ErrorType::type_error("contract".into(), value))?;

        let mut used_fields = HashSet::with_capacity(field_types.len());
        let mut field_values = Vec::with_capacity(field_types.len());
        for (name, r#type) in field_types.into_iter() {
            used_fields.insert(name.clone());

            let json_value = object
                .remove(name.as_str())
                .ok_or_else(|| ErrorType::MissingField(name.clone()))?;

            let value =
                Self::try_from_typed_json(json_value, r#type).push_structure(name.as_str())?;

            field_values.push((name, value));
        }

        for field in object.keys() {
            if !used_fields.contains(field.as_str()) {
                return Err(ErrorType::UnexpectedField(field.clone()).into());
            }
        }

        Ok(Self::Contract(field_values))
    }
}