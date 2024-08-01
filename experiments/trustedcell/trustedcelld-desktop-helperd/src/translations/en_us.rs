use super::Translation;

pub struct Lang;
impl Translation for Lang {
    fn translate_access_vector(&self, access_vector: &crate::AccessVector) -> String {
        if access_vector.object.owner_mode() {
            format!(
                "Do you want to allow {} to access {}'s {} with {:?}?",
                access_vector.subject_cell,
                access_vector.object.owner,
                access_vector.object.category,
                access_vector.action
            )
        } else {
            format!(
                "Do you want to allow {} to access {} with {:?}?",
                access_vector.subject_cell, access_vector.object.category, access_vector.action
            )
        }
    }
}
