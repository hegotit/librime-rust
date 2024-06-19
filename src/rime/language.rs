pub(crate) trait LanguageProvider {
    fn language(&self) -> Option<&Language>;
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Language {
    name: String,
}

impl Language {
    pub(crate) fn new(name: &str) -> Language {
        Language {
            name: name.to_string(),
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn intelligible<T, U>(t: &Option<T>, u: &Option<U>) -> bool
    where
        T: LanguageProvider,
        U: LanguageProvider,
    {
        if let (Some(t_lang), Some(u_lang)) = (t.as_ref(), u.as_ref()) {
            if let (Some(t_language), Some(u_language)) = (t_lang.language(), u_lang.language()) {
                return t_language == u_language;
            }
        }
        false
    }

    pub(crate) fn get_language_component(name: &str) -> String {
        if let Some(dot) = name.find('.') {
            if dot != 0 {
                return name[..dot].to_string();
            }
        }
        name.to_string()
    }
}
