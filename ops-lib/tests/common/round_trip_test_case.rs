use ops_lib::Format;

#[derive(Debug)]
pub struct RoundTripTestCase {
    pub message: String,
    pub given: (Format, String),
}
