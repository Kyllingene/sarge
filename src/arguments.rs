use crate::ArgParseError;

pub trait Arguments {
    fn new() -> Self;
    fn parse_args(&mut self, args: &[String]) -> Result<(), ArgParseError>;
    
    fn parse(&mut self) -> Result<(), ArgParseError> {
        self.parse_args(std::env::args().collect::<Vec<_>>().as_slice())
    }
}
