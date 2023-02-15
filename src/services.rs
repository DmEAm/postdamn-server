use uuid::{Uuid};
use std::fmt::{Display, Formatter, Result};

#[derive(Default, Debug)]
pub struct Example {
    pub id: Uuid,
    pub other_id: Uuid,
}

impl Example {
    pub fn reset(&mut self) {
    }
}

impl Display for Example {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Id={},other-Id={}", self.id, self.other_id)
    }
}