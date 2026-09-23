#[derive(Debug)]
pub struct TestCase<Given, Expected> {
    pub message: String,
    pub given: Given,
    pub expected: Expected,
}
