use std::fmt;

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct VatName(pub String);
impl fmt::Display for VatName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
