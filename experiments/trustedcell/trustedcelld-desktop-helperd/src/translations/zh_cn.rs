use super::Translations;

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
                "要允许 {} 访问 {} 的 {}，以 {:?} 方式吗？",
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
