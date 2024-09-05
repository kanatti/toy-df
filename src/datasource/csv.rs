use arrow_schema::SchemaRef;

pub struct CsvReadOptions {
    pub has_header: bool,
    pub schema: Option<SchemaRef>
}

impl CsvReadOptions {
    pub fn new(has_header: bool, schema: Option<SchemaRef>) -> Self {
        Self { has_header, schema }
    }
}