//!
//! The semantic analyzer structure value element.
//!

mod tests;

pub mod error;

use std::fmt;

use crate::semantic::element::access::AccessData;
use crate::semantic::element::r#type::structure::Structure as StructureType;
use crate::semantic::element::r#type::Type;

use self::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Structure {
    r#type: StructureType,
    field_index: usize,
}

impl Structure {
    pub fn new(r#type: StructureType) -> Self {
        Self {
            r#type,
            field_index: 0,
        }
    }

    pub fn slice(&self, field_name: &str) -> Result<AccessData, Error> {
        let mut offset = 0;
        let total_size = self.r#type().size();

        for (name, r#type) in self.r#type.fields.iter() {
            if name == field_name {
                return Ok(AccessData::new(
                    offset,
                    r#type.size(),
                    total_size,
                    r#type.to_owned(),
                ));
            }
            offset += r#type.size();
        }

        Err(Error::FieldDoesNotExist {
            type_identifier: self.r#type.identifier.to_string(),
            field_name: field_name.to_owned(),
        })
    }

    pub fn r#type(&self) -> Type {
        Type::Structure(self.r#type.to_owned())
    }

    pub fn push(&mut self, name: String, r#type: Type) -> Result<(), Error> {
        match self.r#type.fields.get(self.field_index) {
            Some((expected_name, expected_type)) => {
                if &name != expected_name {
                    return Err(Error::FieldExpected {
                        type_identifier: self.r#type.identifier.to_owned(),
                        position: self.field_index + 1,
                        expected: expected_name.to_owned(),
                        found: name,
                    });
                }
                if &r#type != expected_type {
                    return Err(Error::FieldInvalidType {
                        type_identifier: self.r#type.identifier.to_owned(),
                        field_name: expected_name.to_owned(),
                        expected: expected_type.to_string(),
                        found: r#type.to_string(),
                    });
                }
            }
            None => {
                return Err(Error::FieldOutOfRange {
                    type_identifier: self.r#type.identifier.to_owned(),
                    expected: self.r#type.fields.len(),
                    found: self.field_index + 1,
                });
            }
        }

        self.field_index += 1;
        Ok(())
    }

    pub fn has_the_same_type_as(&self, other: &Self) -> bool {
        self.r#type.unique_id == other.r#type.unique_id
    }
}

impl fmt::Display for Structure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "structure '{}' with fields {}",
            self.r#type.identifier,
            self.r#type
                .fields
                .iter()
                .map(|(name, r#type)| format!("'{}' of type '{}'", name, r#type))
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}
