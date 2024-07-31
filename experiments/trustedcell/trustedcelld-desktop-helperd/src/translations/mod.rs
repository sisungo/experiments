mod en_us;
mod zh_cn;

use crate::AccessVector;

pub trait I18NDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, lang: &dyn Translations) -> std::fmt::Result;
}

pub trait Translations {
    fn translate_access_vector(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        access_vector: &AccessVector,
    ) -> std::fmt::Result;
}
