mod en_us;
mod zh_cn;

pub trait I18NDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, lang: &dyn Translations) -> std::fmt::Result;
}

pub trait Translations {}