mod commons;

#[cfg(test)]
mod tests {
    use librime_rust::rime::common::PathExt;
    use librime_rust::rime::dict::db::{Db, DbAccessor};
    use librime_rust::rime::dict::text_db::TextDb;
    use librime_rust::rime::dict::user_db::UserDbWrapper;

    use crate::commons;

    #[test]
    fn access_record_by_key() {
        commons::enable_log();
        let mut db = UserDbWrapper::<TextDb>::new(PathExt::new("user_db_test.txt"), "user_db_test");
        if db.exists() {
            db.remove();
        }
        assert!(!db.exists());
        db.open();
        assert!(db.loaded());
        assert!(db.update("abc", "ZYX"));
        assert!(db.update("zyx", "CBA"));
        assert!(db.update("zyx", "ABC"));
        let value = db.fetch("abc");
        assert!(value.is_some());
        assert_eq!(Some("ZYX"), value.as_deref());

        let value = db.fetch("zyx");
        assert!(value.is_some());
        assert_eq!(Some("ABC"), value.as_deref());

        assert!(db.fetch("wvu").is_none());

        assert!(db.erase("zyx"));
        assert!(db.fetch("zyx").is_none());

        assert!(db.close());
        assert!(!db.loaded());
    }

    #[test]
    fn query() {
        commons::enable_log();
        let db_path = PathExt::new("user_db_test.txt");
        let mut db = UserDbWrapper::<TextDb>::new(db_path, "user_db_test");

        if db.exists() {
            db.remove();
        }
        assert!(!db.exists());

        db.open();
        assert!(db.update("abc", "ZYX"));
        assert!(db.update("abc\tdef", "ZYX WVU"));
        assert!(db.update("zyx", "ABC"));
        assert!(db.update("wvu", "DEF"));

        // "abc"
        {
            let accessor = db.query(Some("abc"));
            assert!(accessor.is_some());

            let mut accessor = accessor.unwrap();

            if let Some((left, right)) = accessor.next() {
                assert_eq!("abc", left.as_str());
                assert_eq!("ZYX", right.as_str());
            } else {
                panic!("pair did not contain expected values");
            }

            if let Some((left, right)) = accessor.next() {
                assert_eq!("abc\tdef", left.as_str());
                assert_eq!("ZYX WVU", right.as_str());
            } else {
                panic!("pair did not contain expected values");
            }

            assert!(accessor.next().is_none());
        }

        // "wvu\tt"
        {
            let accessor = db.query(Some("wvu\tt"));
            assert!(accessor.is_some());
            assert!(accessor.unwrap().next().is_none());
        }

        // "z"
        {
            let accessor = db.query(Some("z"));
            assert!(accessor.is_some());

            let mut accessor = accessor.unwrap();

            if let Some((left, right)) = accessor.next() {
                assert_eq!("zyx", left.as_str());
                assert_eq!("ABC", right.as_str());
            } else {
                panic!("pair did not contain expected values");
            }

            assert!(accessor.next().is_none());
        }

        db.close();
    }
}
