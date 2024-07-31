use super::Translations;
use crate::Action;
use std::fmt::Write;

pub struct Lang {}
impl Translations for Lang {
    fn translate_access_vector(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        access_vector: &crate::AccessVector,
    ) -> std::fmt::Result {
        if access_vector.object.owner_mode() {
            write!(
                f,
                "Do you want to allow {} to access {}'s {} with {:?}?",
                access_vector.subject_cell,
                access_vector.object.owner,
                access_vector.object.category,
                access_vector.action
            )
        } else {
            write!(f, "")
        }
    }
}
