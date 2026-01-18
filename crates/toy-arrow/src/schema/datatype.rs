#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Boolean,
    Utf8,
    Binary,
    Timestamp(TimeUnit, Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeUnit {
    Second,
    MilliSecond,
    MicroSecond,
    NanoSecond,
}
