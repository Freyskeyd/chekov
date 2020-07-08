#[derive(Debug, PartialEq)]
pub enum ExpectedVersion {
    AnyVersion,
    NoStream,
    StreamExists,
    Version(i64),
}
