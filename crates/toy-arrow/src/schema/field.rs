use crate::schema::DataType;

#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Field {
    pub fn new(name: String, data_type: DataType, nullable: bool) -> Self {
        Field {
            name,
            data_type,
            nullable,
        }
    }
}
