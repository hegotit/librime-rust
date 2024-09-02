#[cfg(test)]
mod tests {
    use librime_rust::rime::key_table::{get_key_name, get_keycode_by_name, Mask, XK_VOID_SYMBOL};
    use librime_rust::rime::key_table::{get_modifier_by_name, get_modifier_name};
    use x11::keysym::{XK_a, XK_grave, XK_space, XK_z, XK_0, XK_A};

    #[test]
    fn keycode_lookup() {
        assert_eq!(Mask::Control as i32, get_modifier_by_name(Some("Control")));
        assert_eq!(0, get_modifier_by_name(Some("abracadabra")));
        assert_eq!(0, get_modifier_by_name(Some("control")));
        assert_eq!(Some("Control"), get_modifier_name(Mask::Control as u32));
        assert_eq!(Some("Release"), get_modifier_name(Mask::Release as u32));
        assert_eq!(
            Some("Control"),
            get_modifier_name((Mask::Control as u32) | (Mask::Release as u32))
        );
    }

    #[test]
    fn modifier_lookup() {
        assert_eq!(XK_A, get_keycode_by_name("A"));
        assert_eq!(XK_z, get_keycode_by_name("z"));
        assert_eq!(XK_0, get_keycode_by_name("0"));
        assert_eq!(XK_grave, get_keycode_by_name("grave"));
        assert_eq!(XK_VOID_SYMBOL, get_keycode_by_name("abracadabra"));
        assert_eq!(XK_VOID_SYMBOL, get_keycode_by_name("Control+c"));
        assert_eq!(Some("a"), get_key_name(XK_a));
        assert_eq!(Some("space"), get_key_name(XK_space));
        assert_eq!(None, get_key_name(0xfffe));
        assert_eq!(None, get_key_name(0xfffffe));
    }
}
