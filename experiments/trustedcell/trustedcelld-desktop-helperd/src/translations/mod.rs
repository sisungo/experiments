mod en_us;
mod zh_cn;

use crate::AccessVector;

pub trait I18NToString {
    fn i18n_to_string(&self, lang: &dyn Translation) -> String;
}

pub trait Translation: Send + Sync {
    fn translate_access_vector(&self, access_vector: &AccessVector) -> String;
}

pub fn lang() -> Box<dyn Translation> {
    match std::env::var("LANG").as_deref() {
        Ok("en_US.UTF-8" | "en-US.UTF-8" | "C.UTF-8" | "C" | "en_US" | "en-US") => {
            Box::new(en_us::Lang)
        }
        Ok("zh_CN.UTF-8" | "zh-CN.UTF-8" | "zh_CN" | "zh-CN") => Box::new(zh_cn::Lang),
        _ => Box::new(en_us::Lang),
    }
}
