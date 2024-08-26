mod commons;

#[cfg(test)]
mod tests {
    use librime_rust::rime::algo::calculus::calculation::Calculus;
    use librime_rust::rime::algo::spelling::{Spelling, SpellingType};

    use crate::commons;

    static TRANSLITERATION: &str = "xlit abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    static TRANSFORMATION: &str = "xform/^([zcs])h(.*)$/$1$2/";
    static ERASURE: &str = "erase/^[czs]h[aoe]ng?$/";
    static DERIVATION: &str = "derive/^([zcs])h/$1/";
    static ABBREVIATION: &str = "abbrev/^([zcs]h).*$/$1/";

    #[test]
    fn transliteration() {
        let calc = Calculus::new();
        let c = calc.parse(TRANSLITERATION);
        assert!(c.is_some());

        let c = c.unwrap();
        let mut s = Spelling::new("abracadabra");
        assert!(c.apply(Some(&mut s)));
        assert_eq!("ABRACADABRA", s.str);
    }

    #[test]
    fn transformation() {
        let calc = Calculus::new();
        let c = calc.parse(TRANSFORMATION);
        assert!(c.is_some());

        let c = c.unwrap();
        let mut s = Spelling::new("shang");
        assert!(c.apply(Some(&mut s)));
        assert_eq!("sang", s.str);
        // non-matching case
        s.str = String::from("bang");
        assert!(!c.apply(Some(&mut s)));
    }

    #[test]
    fn erasure() {
        let calc = Calculus::new();
        let c = calc.parse(ERASURE);
        assert!(c.is_some());

        let c = c.unwrap();
        assert!(!c.addition());
        assert!(c.deletion());

        let mut s = Spelling::new("shang");
        assert!(c.apply(Some(&mut s)));
        assert_eq!("", s.str);
        // non-matching case
        s.str = String::from("bang");
        assert!(!c.apply(Some(&mut s)));
    }

    #[test]
    fn derivation() {
        let calc = Calculus::new();
        let c = calc.parse(DERIVATION);
        assert!(c.is_some());

        let c = c.unwrap();
        assert!(c.addition());
        assert!(!c.deletion());

        let mut s = Spelling::new("shang");
        assert!(c.apply(Some(&mut s)));
        assert_eq!("sang", s.str);
        // non-matching case
        s.str = String::from("bang");
        assert!(!c.apply(Some(&mut s)));
    }

    #[test]
    fn abbreviation() {
        let calc = Calculus::new();
        let c = calc.parse(ABBREVIATION);
        assert!(c.is_some());

        let c = c.unwrap();
        assert!(c.addition());
        assert!(!c.deletion());

        let mut s = Spelling::new("shang");
        assert!(c.apply(Some(&mut s)));
        assert_eq!("sh", s.str);
        assert_eq!(SpellingType::Abbreviation, s.properties.type_);
        assert!(commons::approx_equal(0.5f64.ln(), s.properties.credibility));
    }
}
