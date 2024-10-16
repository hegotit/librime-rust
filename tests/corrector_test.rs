mod commons;

#[cfg(test)]
mod tests {
    use librime_rust::rime::algo::spelling::SpellingType;
    use librime_rust::rime::algo::syllabifier::{Syllabifier, SyllableGraph};
    use librime_rust::rime::algo::SyllableId;
    use librime_rust::rime::common::PathExt;
    use librime_rust::rime::dict::corrector::NearSearchCorrector;
    use librime_rust::rime::dict::prism::Prism;

    use log::info;

    use std::collections::BTreeMap;
    use std::sync::LazyLock;

    use crate::commons;

    static RIME_CORRECTOR_SEARCH: LazyLock<(Prism, BTreeMap<String, SyllableId>)> =
        LazyLock::new(|| {
            let mut syllables = vec!["chang", "tuan"];
            syllables.sort();

            let syllable_id = syllables
                .iter()
                .enumerate()
                .map(|(i, &syllable)| (syllable.to_owned(), i as SyllableId))
                .collect();

            let file_path = PathExt::new("corrector_simple_test.prism.bin");
            let mut prism = Prism::new(file_path);

            let keyset = syllables.into_iter().collect();
            prism.build(&keyset);

            (prism, syllable_id)
        });

    static RIME_CORRECTOR: LazyLock<(Prism, BTreeMap<String, SyllableId>)> = LazyLock::new(|| {
        let mut syllables = vec!["j", "ji", "jie", "ju", "jue", "shen"];
        syllables.sort();

        let syllable_id = syllables
            .iter()
            .enumerate()
            .map(|(i, &syllable)| (syllable.to_owned(), i as SyllableId))
            .collect();

        let file_path = PathExt::new("corrector_test.prism.bin");
        let mut prism = Prism::new(file_path);

        let keyset = syllables.into_iter().collect();
        prism.build(&keyset);

        (prism, syllable_id)
    });

    fn setup_prism_for_search() -> &'static (Prism, BTreeMap<String, SyllableId>) {
        &RIME_CORRECTOR_SEARCH
    }

    fn setup_prism() -> &'static (Prism, BTreeMap<String, SyllableId>) {
        &RIME_CORRECTOR
    }

    #[test]
    fn case_near_substitute() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism_for_search();
        let mut syllabifier = Syllabifier::default();
        let mut corrector = NearSearchCorrector::default();
        syllabifier.enable_correction(&mut corrector);

        let mut graph = SyllableGraph::default();
        let input = "chsng";
        syllabifier.build_syllable_graph(input, &prism, &mut graph);

        assert_eq!(input.len(), graph.input_length());
        assert_eq!(input.len(), graph.interpreted_length());

        assert_eq!(2, graph.vertices().len());
        assert!(graph.vertices().get(&5).is_some());

        let sp = &graph.edges()[&0][&5];
        assert_eq!(1, sp.len());
        assert!(sp.get(&syllable_id["chang"]).is_some());
    }

    #[test]
    fn case_far_substitute() {
        commons::enable_log();
        let (prism, _) = setup_prism_for_search();
        let mut syllabifier = Syllabifier::default();
        let mut corrector = NearSearchCorrector::default();
        syllabifier.enable_correction(&mut corrector);

        let mut graph = SyllableGraph::default();
        let input = "chpng";
        syllabifier.build_syllable_graph(input, &prism, &mut graph);

        assert_eq!(input.len(), graph.input_length());
        assert_eq!(0, graph.interpreted_length());
        assert_eq!(1, graph.vertices().len());
        assert!(graph.vertices().get(&5).is_none());
    }

    #[test]
    #[ignore]
    fn case_transpose() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism_for_search();
        let mut syllabifier = Syllabifier::default();
        let mut corrector = NearSearchCorrector::default();
        syllabifier.enable_correction(&mut corrector);

        let mut graph = SyllableGraph::default();
        let input = "cahng";
        syllabifier.build_syllable_graph(input, &prism, &mut graph);

        assert_eq!(input.len(), graph.input_length());
        assert_eq!(input.len(), graph.interpreted_length());
        assert_eq!(2, graph.vertices().len());
        assert!(graph.vertices().get(&5).is_some());

        let sp = &graph.edges()[&0][&5];
        assert_eq!(1, sp.len());
        assert!(sp.get(&syllable_id["chang"]).is_some());
    }

    #[test]
    fn case_correction_syllabify() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism_for_search();
        let mut syllabifier = Syllabifier::default();
        let mut corrector = NearSearchCorrector::default();
        syllabifier.enable_correction(&mut corrector);

        let mut graph = SyllableGraph::default();
        let input = "chabgtyan";
        syllabifier.build_syllable_graph(input, &prism, &mut graph);

        assert_eq!(input.len(), graph.input_length());
        assert_eq!(input.len(), graph.interpreted_length());
        assert_eq!(3, graph.vertices().len());
        assert!(graph.vertices().get(&9).is_some());

        let sp1 = &graph.edges()[&0][&5];
        assert_eq!(1, sp1.len());
        assert!(sp1.get(&syllable_id["chang"]).is_some());
        assert!(sp1[&0].is_correction());

        let sp2 = &graph.edges()[&5][&9];
        assert_eq!(1, sp2.len());
        assert!(sp2.get(&syllable_id["tuan"]).is_some());
        assert!(sp2[&1].is_correction());
    }

    #[test]
    fn case_multiple_edges1() {
        commons::enable_log();
        let (prism, syllable_id) = setup_prism();
        let mut syllabifier = Syllabifier::default();
        let mut corrector = NearSearchCorrector::default();
        syllabifier.enable_correction(&mut corrector);

        let mut graph = SyllableGraph::default();
        let input = "jiejue"; // jie'jue jie'jie jue'jue jue'jie
        syllabifier.build_syllable_graph(input, &prism, &mut graph);

        assert_eq!(input.len(), graph.input_length());
        assert_eq!(input.len(), graph.interpreted_length());

        info!("graph.edges(): {:#?}", graph);

        let sp1 = &graph.edges()[&0][&3];
        assert_eq!(2, sp1.len());
        assert!(sp1.get(&syllable_id["jie"]).is_some());
        assert_eq!(
            sp1.get(&syllable_id["jie"]).unwrap().type_,
            SpellingType::Normal
        );
        assert!(sp1.get(&syllable_id["jue"]).is_some());
        assert!(sp1.get(&syllable_id["jue"]).unwrap().is_correction());

        let sp2 = &graph.edges()[&3][&6];
        assert_eq!(2, sp2.len());
        assert!(sp2.get(&syllable_id["jie"]).is_some());
        assert!(sp2.get(&syllable_id["jie"]).unwrap().is_correction());
        assert!(sp2.get(&syllable_id["jue"]).is_some());
        assert_eq!(
            sp2.get(&syllable_id["jue"]).unwrap().type_,
            SpellingType::Normal
        );
    }
}
