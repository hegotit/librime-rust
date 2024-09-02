#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use librime_rust::rime::key_event::{KeyEvent, KeySequence};
    use librime_rust::rime::key_table::Mask;
    use x11::keysym::{
        XK_Return, XK_a, XK_comma, XK_less, XK_period, XK_plus, XK_space, XK_z, XK_A, XK_F4,
    };

    #[test]
    fn key_name() {
        let comma = KeyEvent::new(XK_comma, 0);
        assert_eq!("comma", comma.repr().as_str());

        let period = KeyEvent::new(XK_period, 0);
        assert_eq!("period", period.repr().as_str());
    }

    #[test]
    fn hex_key_name() {
        let ke_fffe = KeyEvent::new(0xfffe, 0);
        assert_eq!("0xfffe", ke_fffe.repr().as_str());

        let ke_fffffe = KeyEvent::new(0xfffffe, 0);
        assert_eq!("0xfffffe", ke_fffffe.repr().as_str());
    }

    #[test]
    fn bad_key_name() {
        let bad_ke = KeyEvent::new(0x1000000, 0);
        assert_eq!("(unknown)", bad_ke.repr().as_str());
    }

    #[test]
    fn modified_key_event() {
        let ke = KeyEvent::new(
            XK_A,
            (Mask::Shift as i32) | (Mask::Control as i32) | (Mask::Release as i32),
        );
        assert!(ke.shift());
        assert!(!ke.alt());
        assert!(ke.ctrl());
        assert!(ke.release());
    }

    #[test]
    fn modified_key_event_representation() {
        let ctrl_a = KeyEvent::new(XK_a, Mask::Control as i32);
        assert_eq!("Control+a", ctrl_a.repr().as_str());

        let less_keyup = KeyEvent::new(XK_less, (Mask::Shift as i32) | (Mask::Release as i32));
        assert_eq!("Shift+Release+less", less_keyup.repr().as_str());
    }

    #[test]
    fn parse_key_event_representation() {
        let mut ke = KeyEvent::default();
        assert!(ke.parse("Shift+Control+Release+A"));
        assert_eq!(XK_A, ke.keycode());
        assert!(ke.shift());
        assert!(!ke.alt());
        assert!(ke.ctrl());
        assert!(ke.release());
    }

    #[test]
    fn construct_key_event_from_representation() {
        let ke = KeyEvent::from_str("Alt+F4").unwrap_or_default();
        assert_eq!(XK_F4, ke.keycode());
        assert!(ke.alt());
        assert!(!ke.release());
    }

    #[test]
    fn equality() {
        let ke0 = KeyEvent::new(XK_plus, 0);
        let ke1 = KeyEvent::from_str("+").unwrap_or_default();
        let ke2 = KeyEvent::from_str("plus").unwrap_or_default();
        assert_eq!(ke0, ke1);
        assert_eq!(ke1, ke2);
    }

    #[test]
    fn serialization() {
        let ke = KeyEvent::new(XK_comma, Mask::Control as i32);
        let serialized = format!("{}", ke);
        assert_eq!("Control+comma", serialized);
    }

    #[test]
    fn plain_string_key_sequence() {
        let ks = KeySequence::from_str("zyx123CBA").unwrap_or_default();
        assert_eq!(9, ks.len());
        assert_eq!(XK_z, ks[0].keycode());
        assert!(!ks[0].release());
        // explanations:
        // Shift is not necessarily implied by uppercase letter 'A'
        // imagine that we may have customized a keyboard with a separate 'A' key
        // when the Shift modifier counts, we'll write "{Shift+A}" instead
        // a real life key sequence could even be like this:
        // "{Shift_L}{Shift+A}{Shift+Release+A}{Shift_L+Release}"
        // here we just focus on the information useful to the ime
        assert_eq!(XK_A, ks[8].keycode());
        assert!(!ks[8].shift());
    }

    #[test]
    fn key_sequence_with_named_keys() {
        let ks = KeySequence::from_str("zyx 123{space}ABC{Return}").unwrap_or_default();
        assert_eq!(12, ks.len());
        assert_eq!(XK_space, ks[3].keycode());
        assert_eq!(XK_space, ks[7].keycode());
        assert_eq!(XK_Return, ks[11].keycode());
    }

    #[test]
    fn key_sequence_with_modified_keys() {
        let ks = KeySequence::from_str("zyx 123{Shift+space}ABC{Control+Alt+Return}")
            .unwrap_or_default();
        assert_eq!(12, ks.len());
        assert_eq!(XK_space, ks[3].keycode());
        assert!(!ks[3].shift());
        assert!(!ks[3].release());
        assert_eq!(XK_space, ks[7].keycode());
        assert!(ks[7].shift());
        assert!(!ks[7].release());
        assert_eq!(XK_Return, ks[11].keycode());
        assert!(!ks[11].shift());
        assert!(ks[11].ctrl());
        assert!(ks[11].alt());
        assert!(!ks[11].release());
    }

    #[test]
    fn stringification() {
        let mut ks = KeySequence::default();
        assert!(ks.parse("z y,x."));
        ks.push(KeyEvent::from_str("{").unwrap_or_default());
        ks.push(KeyEvent::from_str("}").unwrap_or_default());
        assert_eq!("z y,x.{braceleft}{braceright}", ks.repr().as_str());
    }

    #[test]
    fn key_sequence_serialization() {
        let ks = KeySequence::from_str("abc, defg.").unwrap_or_default();
        let serialized = format!("{}", ks);
        assert_eq!("abc, defg.", serialized);
    }
}
