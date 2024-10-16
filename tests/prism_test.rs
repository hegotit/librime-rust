mod commons;

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use librime_rust::rime::common::PathExt;
    use librime_rust::rime::dict::prism::Prism;

    use crate::commons;

    struct RimePrismTest {
        prism: Prism,
    }

    impl RimePrismTest {
        fn new() -> Self {
            let mut prism = Prism::new(PathExt::new("prism_test.bin"));
            //prism.remove();

            let mut keyset = BTreeSet::new();
            keyset.insert("google"); // 4
            keyset.insert("good"); // 2
            keyset.insert("goodbye"); // 3
            keyset.insert("microsoft");
            keyset.insert("macrosoft");
            keyset.insert("adobe"); // 0 == id
            keyset.insert("yahoo");
            keyset.insert("baidu"); // 1

            prism.build(&keyset);
            Self { prism }
        }
    }

    #[test]
    #[ignore]
    fn save_and_load() {
        let test_obj = RimePrismTest::new();
        assert!(test_obj.prism.save());

        //let mut test_prism = Prism::new(test_obj.prism.file_path());
        //assert!(test_prism.load());

        //assert_eq!(test_obj.prism.array_size(), test_prism.array_size());
    }

    #[test]
    fn has_key() {
        let test_obj = RimePrismTest::new();
        assert!(test_obj.prism.has_key("google"));
        assert!(!test_obj.prism.has_key("googlesoft"));

        assert!(test_obj.prism.has_key("microsoft"));
        assert!(!test_obj.prism.has_key("peoplesoft"));
    }

    #[test]
    fn get_value() {
        let test_obj = RimePrismTest::new();
        assert_eq!(test_obj.prism.get_value("adobe"), Some(0));
        assert_eq!(test_obj.prism.get_value("baidu"), Some(1));
    }

    #[test]
    fn common_prefix_match() {
        let test_obj = RimePrismTest::new();
        let result = test_obj
            .prism
            .common_prefix_search("goodbye")
            .expect("No result meeted");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].value(), 2); // good
        assert_eq!(result[0].offset(), 4); // good
        assert_eq!(result[1].value(), 3); // goodbye
        assert_eq!(result[1].offset(), 7); // goodbye
    }

    #[test]
    fn expand_search() {
        commons::enable_log();
        let test_obj = RimePrismTest::new();
        let result = test_obj
            .prism
            .expand_search("goo", 10)
            .expect("No result meeted");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].value(), 2); // good
        assert_eq!(result[0].offset(), 4); // good
        assert_eq!(result[1].value(), 4); // google
        assert_eq!(result[1].offset(), 6); // google
        assert_eq!(result[2].value(), 3); // goodbye
        assert_eq!(result[2].offset(), 7); // goodbye
    }
}
