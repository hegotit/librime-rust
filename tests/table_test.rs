//#[cfg(test)]
//mod tests {
//    use librime_rust::rime::common::PathExt;
//    use librime_rust::rime::dict::table::Table;
//    use librime_rust::rime::dict::vocabulary::{
//        ShortDictEntry, Syllabary, Vocabulary, VocabularyPage,
//    };

//    fn prepare_sample_vocabulary() -> (Syllabary, Vocabulary) {
//        let mut syll = Syllabary::new();
//        let mut voc = Vocabulary::default();

//        syll.insert("0".to_string()); // no entries for '0', however
//        syll.insert("1".to_string());
//        syll.insert("2".to_string());
//        syll.insert("3".to_string());
//        syll.insert("4".to_string());

//        let mut entry = ShortDictEntry::new("yi", vec![1], 1.0);
//        voc.entry(1).or_default().entries.push(entry.clone());

//        entry.code[0] = 2;
//        entry.text = "er".to_string();
//        voc.entry(2).or_default().entries.push(entry.clone());

//        entry.text = "liang".to_string();
//        voc.entry(2).or_default().entries.push(entry.clone());

//        entry.text = "lia".to_string();
//        voc.entry(2).or_default().entries.push(entry.clone());

//        entry.code[0] = 3;
//        entry.text = "san".to_string();
//        voc.entry(3).or_default().entries.push(entry.clone());

//        entry.text = "sa".to_string();
//        voc.entry(3).or_default().entries.push(entry);

//        let mut lv2 = Vocabulary::default();
//        let mut lv3 = Vocabulary::default();
//        let mut lv4 = Vocabulary::default();

//        let short1 = ShortDictEntry::new("yi-er-san-si", vec![1, 2, 3, 4], 1.0);
//        let short2 = ShortDictEntry::new("yi-er-san-er-yi", vec![1, 2, 3, 2, 1], 1.0);
//        lv4.insert(-1, VocabularyPage::new(vec![short1, short2]));

//        let page = lv3.entry(3).or_default();
//        page.next_level = Some(lv4);
//        page.entries
//            .push(ShortDictEntry::new("yi-er-san", vec![1, 2, 3], 1.0));

//        lv2.entry(2).or_default().next_level = Some(lv3);

//        voc.get_mut(&1).unwrap().next_level = Some(lv2);

//        (syll, voc)
//    }

//    struct TableTest {
//        table: Table,
//    }

//    impl TableTest {
//        fn new() -> Self {
//            let mut table = Table::new(PathExt::new("table_test.bin"));
//            let (syll, voc) = prepare_sample_vocabulary();
//            assert!(table.build(&syll, &voc, 8, 0));
//            //assert!(table.save());
//            //table.load();
//            //TableTest { table }
//            todo!()
//        }
//    }

//    //#[test]
//    //fn integrity_test() {
//    //    let table_test = TableTest::new();
//    //    assert!(table_test.table.load());
//    //}

//    #[test]
//    fn simple_query_test() {
//        let table_test = TableTest::new();

//        assert_eq!(Some("0"), table_test.table.get_syllable_by_id(0));
//        assert_eq!(Some("3"), table_test.table.get_syllable_by_id(3));
//        assert_eq!(Some("4"), table_test.table.get_syllable_by_id(4));

//        if let Some(v) = table_test.table.query_words(1) {
//            assert_eq!(v.len(), 1);
//            assert_eq!(v[0].text, "yi");
//            assert_eq!(v[0].weight, 1.0);
//        }

//        if let Some(v) = table_test.table.query_words(2) {
//            assert_eq!(v.len(), 3);
//            assert_eq!(v[0].text, "er");
//            assert_eq!(v[1].text, "liang");
//            assert_eq!(v[2].text, "lia");
//        }

//        if let Some(v) = table_test.table.query_words(3) {
//            assert_eq!(v.len(), 2);
//            assert_eq!(v[0].text, "san");
//            assert_eq!(v[1].text, "sa");
//        }

//        let code = vec![1, 2, 3];
//        if let Some(v) = table_test.table.query_phrases(&code) {
//            assert_eq!(v.len(), 1);
//            assert_eq!(v[0].text, "yi-er-san");
//        }
//    }
//}
