#[derive(Debug, Clone, PartialEq)]
pub struct AddressBook {
    pub ab: String,
}

impl AddressBook {
    pub fn empty() -> Self {
        Self { 
            ab: "{}".to_string() 
        }
    }
}

