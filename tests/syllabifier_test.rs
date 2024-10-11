mod commons;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::LazyLock;

    use librime_rust::rime::algo::spelling::SpellingType;
    use librime_rust::rime::algo::syllabifier::{Syllabifier, SyllableGraph};
    use librime_rust::rime::algo::SyllableId;
    use librime_rust::rime::common::PathExt;
    use librime_rust::rime::dict::corrector::NearSearchCorrector;
    use librime_rust::rime::dict::prism::Prism;

    use crate::commons;

    static RIME_SYLLABIFIER: LazyLock<(Prism, BTreeMap<String, SyllableId>)> =
        LazyLock::new(|| {
            let mut syllables = vec![
                "a", "an", "cha", "chan", "chang", "gan", "han", "hang", "na", "tu", "tuan",
            ];
            syllables.sort();

            let syllable_id = syllables
                .iter()
                .enumerate()
                .map(|(i, &syllable)| (syllable.to_owned(), i as SyllableId))
                .collect();

            let file_path = PathExt::new("syllabifier_test.bin");
            let mut prism = Prism::new(file_path);
            let keyset = syllables.into_iter().collect();
            prism.build(&keyset);

            (prism, syllable_id)
        });

    fn setup_prism() -> &'static (Prism, BTreeMap<String, SyllableId>) {
        &RIME_SYLLABIFIER
    }

    #[test]
    fn case_alpha() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism();
        let s: Syllabifier<'_, NearSearchCorrector> = Syllabifier::default();
        let mut g = SyllableGraph::default();
        let input = "a";
        s.build_syllable_graph(input, &prism, &mut g);

        assert_eq!(input.len(), g.input_length());
        assert_eq!(input.len(), g.interpreted_length());
        assert_eq!(2, g.vertices().len());
        assert!(g.vertices().get(&1).is_some());
        assert_eq!(SpellingType::Normal, g.vertices()[&1]);

        let sp = &g.edges()[&0][&1];
        assert_eq!(1, sp.len());
        let key = syllable_id["a"];
        assert!(sp.get(&key).is_some());
        assert_eq!(SpellingType::Normal, sp[&0].type_);
        assert_eq!(0.0, sp[&0].credibility);
    }

    #[test]
    fn case_failure() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism();
        let s: Syllabifier<'_, NearSearchCorrector> = Syllabifier::default();
        let mut g = SyllableGraph::default();
        let input = "ang";
        s.build_syllable_graph(input, &prism, &mut g);

        assert_eq!(input.len(), g.input_length());
        assert_eq!(input.len() - 1, g.interpreted_length());
        assert_eq!(2, g.vertices().len());
        assert!(g.vertices().get(&1).is_none());
        assert!(g.vertices().get(&2).is_some());
        assert_eq!(SpellingType::Normal, g.vertices()[&2]);

        let sp = &g.edges()[&0][&2];
        assert_eq!(1, sp.len());
        assert!(sp.get(&syllable_id["an"]).is_some());
    }

    #[test]
    fn case_changan() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism();
        let s: Syllabifier<'_, NearSearchCorrector> = Syllabifier::default();
        let mut g = SyllableGraph::default();
        let input = "changan";
        s.build_syllable_graph(input, &prism, &mut g);

        assert_eq!(input.len(), g.input_length());
        assert_eq!(input.len(), g.interpreted_length());
        assert_eq!(4, g.vertices().len());
        // not c'han'gan or c'hang'an
        assert!(g.vertices().get(&1).is_none());
        assert!(g.vertices().get(&4).is_some());
        assert!(g.vertices().get(&5).is_some());
        assert_eq!(SpellingType::Normal, g.vertices()[&4]);
        assert_eq!(SpellingType::Normal, g.vertices()[&5]);

        // chan, chang but not cha
        let e0 = &g.edges()[&0];
        assert_eq!(2, e0.len());
        assert!(e0.get(&4).is_some());
        assert!(e0.get(&5).is_some());
        assert!(e0[&4].get(&syllable_id["chan"]).is_some());
        assert!(e0[&5].get(&syllable_id["chang"]).is_some());

        // gan$
        let e4 = &g.edges()[&4];
        assert_eq!(1, e4.len());
        assert!(e4.get(&7).is_some());
        assert!(e4[&7].get(&syllable_id["gan"]).is_some());

        // an$
        let e5 = &g.edges()[&5];
        assert_eq!(1, e5.len());
        assert!(e5.get(&7).is_some());
        assert!(e5[&7].get(&syllable_id["an"]).is_some());
    }

    #[test]
    fn case_tuan() {
        let (prism, syllable_id) = setup_prism();
        let s: Syllabifier<'_, NearSearchCorrector> = Syllabifier::default();
        let mut g = SyllableGraph::default();
        let input = "tuan";
        s.build_syllable_graph(input, &prism, &mut g);

        assert_eq!(input.len(), g.input_length());
        assert_eq!(input.len(), g.interpreted_length());
        assert_eq!(3, g.vertices().len());

        // both tu'an and tuan
        assert!(g.vertices().get(&2).is_some());
        assert!(g.vertices().get(&4).is_some());
        assert_eq!(SpellingType::Ambiguous, g.vertices()[&2]);
        assert_eq!(SpellingType::Normal, g.vertices()[&4]);

        let e0 = &g.edges()[&0];
        assert_eq!(2, e0.len());
        assert!(e0.get(&2).is_some());
        assert!(e0.get(&4).is_some());
        assert!(e0[&2].get(&syllable_id["tu"]).is_some());
        assert!(e0[&4].get(&syllable_id["tuan"]).is_some());

        // an$
        let e2 = &g.edges()[&2];
        assert_eq!(1, e2.len());
        assert!(e2.get(&4).is_some());
        assert!(e2[&4].get(&syllable_id["an"]).is_some());
    }

    #[test]
    fn case_chaining_ambiguity() {
        let (prism, _syllable_id) = setup_prism();
        let s: Syllabifier<'_, NearSearchCorrector> = Syllabifier::default();
        let mut g = SyllableGraph::default();
        let input = "anana";
        s.build_syllable_graph(input, &prism, &mut g);

        assert_eq!(input.len(), g.input_length());
        assert_eq!(input.len(), g.interpreted_length());
        assert_eq!(input.len() + 1, g.vertices().len());
    }

    #[test]
    fn transposed_syllable_graph() {
        let (prism, syllable_id) = setup_prism();
        let s: Syllabifier<'_, NearSearchCorrector> = Syllabifier::default();
        let mut g = SyllableGraph::default();
        let input = "changan";
        s.build_syllable_graph(input, &prism, &mut g);

        assert!(g.indices().get(&0).is_some());
        assert_eq!(2, g.indices()[&0].len());
        assert!(g.indices()[&0].get(&syllable_id["chan"]).is_some());
        assert!(g.indices()[&0].get(&syllable_id["chang"]).is_some());

        let chan_indices = &g.indices()[&0][&syllable_id["chan"]];
        assert_eq!(1, chan_indices.len());
        assert!(chan_indices.get(0).is_some());
        assert_eq!(4, chan_indices[0].end_pos);
    }
}
