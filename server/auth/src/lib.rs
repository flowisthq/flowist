/// The token's Subject claim, which corresponds with the username
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Subject(pub Option<String>);
