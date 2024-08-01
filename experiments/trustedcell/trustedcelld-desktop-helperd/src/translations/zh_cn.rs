use super::Translation;

pub struct Lang;
impl Translation for Lang {
    fn translate_access_vector(&self, access_vector: &crate::AccessVector) -> String {
        if access_vector.object.owner_mode() {
            format!(
                "要允许 {} 访问 {} 的 {}，以 {:?} 方式吗？",
                access_vector.subject_cell,
                access_vector.object.owner,
                access_vector.object.category,
                access_vector.action
            )
        } else {
            format!(
                "要允许 {} 访问 {}，以 {:?} 方式吗？",
                access_vector.subject_cell, access_vector.object.category, access_vector.action
            )
        }
    }
}
