use crate::{Attributes, Result};

pub trait AttributeOperations {
    fn get_attributes(
        &self,
        context: &mut crate::Context,
        attributes: &mut Attributes,
    ) -> Result<()>;

    fn set_attributes(&self, context: &mut crate::Context, attributes: &Attributes) -> Result<()>;
}
