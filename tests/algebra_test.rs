mod commons;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use librime_rust::rime::algo::algebra::{Projection, Script};
    use librime_rust::rime::algo::spelling::SpellingType;
    use librime_rust::rime::config::config_types::{ConfigList, ConfigValue};

    use crate::commons;

    static TRANSLITERATION: &str = "xlit/ABCDEFGHIJKLMNOPQRSTUVWXYZ/abcdefghijklmnopqrstuvwxyz/";
    static TRANSFORMATION: &str = "xform/^([zcs])h(.*)$/$1$2/";

    static NUM_OF_INSTRUCTIONS: usize = 5;
    static INSTRUCTIONS: [&str; 5] = [
        "xform/^([a-z]+)\\d$/$1/",
        "erase/^[wxy].*$/",
        "derive/^([zcs])h(.*)$/$1$2/",
        "abbrev/^([a-z]).+$/$1/",
        "abbrev/^([zcs]h).+$/$1/",
    ];

    #[test]
    fn spelling_manipulation() {
        commons::enable_log();
        let mut c = ConfigList::new();
        c.append(Some(Arc::new(ConfigValue::from_str(TRANSLITERATION))));
        c.append(Some(Arc::new(ConfigValue::from_str(TRANSFORMATION))));

        let mut p = Projection::new();
        assert!(p.load(Some(Arc::new(c))));

        let mut str = String::from("Shang");
        assert!(p.apply(Some(&mut str)));
        assert_eq!(str, "sang");
    }

    #[test]
    fn projection() {
        commons::enable_log();
        let mut c = ConfigList::new();
        for i in 0..NUM_OF_INSTRUCTIONS {
            c.append(Some(Arc::new(ConfigValue::from_str(INSTRUCTIONS[i]))));
        }

        let mut p = Projection::new();
        assert!(p.load(Some(Arc::new(c))));

        let mut s = Script::new();
        s.add_syllable("zhang1");
        s.add_syllable("chang2");
        s.add_syllable("shang3");
        s.add_syllable("shang4");
        s.add_syllable("zang1");
        s.add_syllable("cang2");
        s.add_syllable("bang3");
        s.add_syllable("wang4");
        assert!(p.apply_script(Some(&mut s)));
        assert_eq!(14, s.len());
        assert!(s.contains_key("chang"));
        assert!(s.contains_key("shang"));
        assert!(s.contains_key("zang"));
        assert!(s.contains_key("cang"));
        assert!(s.contains_key("sang"));
        assert!(s.contains_key("bang"));
        assert!(s.contains_key("zh"));
        assert!(s.contains_key("ch"));
        assert!(s.contains_key("sh"));
        assert!(s.contains_key("z"));
        assert!(s.contains_key("c"));
        assert!(s.contains_key("s"));
        assert!(s.contains_key("b"));

        assert!(!s.contains_key("wang"));
        assert!(!s.contains_key("wang4"));
        assert!(!s.contains_key("zhang1"));
        assert!(!s.contains_key("bang3"));

        assert_eq!(2, s["z"].len());
        assert_eq!(1, s["zh"].len());
        assert_eq!(2, s["sh"].len());

        assert_eq!(SpellingType::Abbreviation, s["sh"][0].properties.type_);
        assert!(commons::approx_equal(
            0.5f64.ln(),
            s["sh"][0].properties.credibility
        ));
    }
}
