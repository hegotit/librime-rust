use crate::rime::algo::spelling::{SpellingProperties, SpellingType};
use crate::rime::algo::SyllableId;
use crate::rime::dict::corrector::{Corrections, Corrector};
use crate::rime::dict::prism::{Match, Prism};
use log::info;
use std::cmp::{max, min, Reverse};
use std::collections::{BTreeMap, BinaryHeap, HashSet};
use std::f64::consts::LN_2;

const COMPLETION_PENALTY: f64 = -LN_2;
const CORRECTION_CREDIBILITY: f64 = -4.605170185988091; // ln(0.01)
const PENALTY_FOR_AMBIGUOUS_SYLLABLE: f64 = -23.025850929940457; // ln(1e-10)

type Vertex = (usize, SpellingType);
struct VertexQueue(BinaryHeap<Reverse<Vertex>>);

impl VertexQueue {
    fn new() -> Self {
        Self(BinaryHeap::new())
    }

    fn push(&mut self, item: Vertex) {
        self.0.push(Reverse(item));
    }

    fn pop(&mut self) -> Option<Vertex> {
        self.0.pop().map(|Reverse(item)| item)
    }

    fn peek(&self) -> Option<&Vertex> {
        self.0.peek().map(|Reverse(item)| item)
    }
}

#[derive(Default)]
struct EdgeProperties {
    sup: SpellingProperties,
    is_correction: bool,
}

impl EdgeProperties {
    fn new(sup: SpellingProperties) -> Self {
        EdgeProperties {
            sup,
            is_correction: false,
        }
    }
}

type VertexMap = BTreeMap<usize, SpellingType>;
type SpellingMap = BTreeMap<SyllableId, EdgeProperties>;
type EndVertexMap = BTreeMap<usize, SpellingMap>;
type EdgeMap = BTreeMap<usize, EndVertexMap>;

type SpellingPropertiesList = Vec<*const EdgeProperties>;
type SpellingIndex = BTreeMap<SyllableId, SpellingPropertiesList>;
type SpellingIndices = BTreeMap<usize, SpellingIndex>;

struct SyllableGraph {
    input_length: usize,
    interpreted_length: usize,
    vertices: VertexMap,
    edges: EdgeMap,
    indices: SpellingIndices,
}

impl Default for SyllableGraph {
    fn default() -> Self {
        SyllableGraph {
            input_length: 0,
            interpreted_length: 0,
            vertices: VertexMap::new(),
            edges: EdgeMap::new(),
            indices: SpellingIndices::new(),
        }
    }
}

struct Syllabifier {
    delimiters: String,
    enable_completion: bool,
    strict_spelling: bool,
    corrector: Option<*mut Corrector>,
}

impl Syllabifier {
    fn new(delimiters: String, enable_completion: bool, strict_spelling: bool) -> Self {
        Syllabifier {
            delimiters,
            enable_completion,
            strict_spelling,
            corrector: None,
        }
    }

    fn build_syllable_graph(
        &self,
        input: &str,
        prism: &mut Prism,
        graph: &mut SyllableGraph,
    ) -> usize {
        if input.is_empty() {
            return 0;
        }

        let mut farthest = 0;
        let mut queue = VertexQueue::new();
        queue.push((0, SpellingType::Normal)); // Start

        while let Some((current_pos, current_type)) = queue.pop() {
            // Record a visit to the vertex
            if !graph.vertices.contains_key(&current_pos) {
                graph.vertices.insert(current_pos, current_type); // Preferred spelling type comes first
            } else {
                continue; // Discard worse spelling types
            }

            if current_pos > farthest {
                farthest = current_pos;
            }
            info!("current_pos: {}", current_pos);

            // See where we can go by advancing a syllable
            let current_input = &input[current_pos..];
            let mut matches = Vec::new();
            let mut exact_match_syllables = HashSet::new();
            prism.common_prefix_search(current_input, &mut matches);

            if let Some(corrector) = self.corrector {
                for m in &matches {
                    exact_match_syllables.insert(m.value);
                }
                let mut corrections = Corrections::new();
                unsafe {
                    (*corrector).tolerance_search(prism, current_input, &mut corrections, 5);
                }
                for m in corrections.0 {
                    let accessor = prism.query_spelling(m.0);
                    if let Some(list) = accessor.spelling_map.get(m.0 as usize) {
                        for item in list {
                            if item.type_ as usize == SpellingType::Normal as usize {
                                matches.push(Match {
                                    value: m.0,
                                    length: m.1.length,
                                });
                                break;
                            }
                        }
                    }
                }
            }

            if !matches.is_empty() {
                let end_vertices = graph.edges.entry(current_pos).or_default();
                for m in matches {
                    if m.length == 0 {
                        continue;
                    }
                    let mut end_pos = current_pos + m.length;
                    // Consume trailing delimiters
                    while end_pos < input.len()
                        && self
                            .delimiters
                            .as_bytes()
                            .contains(&input.as_bytes()[end_pos])
                    {
                        end_pos += 1;
                    }
                    info!("end_pos: {}", end_pos);
                    let matches_input = current_pos == 0 && end_pos == input.len();
                    let spellings = end_vertices.entry(end_pos).or_default();
                    let mut end_vertex_type = SpellingType::Invalid;
                    /*
                        When spelling algebra is enabled, a spelling evaluates to a set of syllables;
                        otherwise, it resembles exactly the syllable itself.
                    */
                    let accessor = prism.query_spelling(m.value);
                    if let Some(list) = accessor.spelling_map.get(m.value as usize) {
                        for item in list {
                            let mut props = EdgeProperties::new(accessor.properties());
                            if self.strict_spelling
                                && matches_input
                                && props.sup.type_ != SpellingType::Normal
                            {
                                // Disqualify fuzzy spelling or abbreviation as single word
                                continue;
                            }
                            props.sup.end_pos = end_pos;
                            // Add a syllable with properties to the edge's
                            // spelling-to-syllable map
                            if self.corrector.is_some() && !exact_match_syllables.contains(&m.value)
                            {
                                props.is_correction = true;
                                props.sup.credibility = CORRECTION_CREDIBILITY;
                            }
                            // Let end_vertex_type be the best (smaller) type of spelling
                            // that ends at the vertex
                            if end_vertex_type > props.sup.type_ && !props.is_correction {
                                end_vertex_type = props.sup.type_;
                            }
                            if let Some(existing) = spellings.get_mut(&item.syllable_id) {
                                existing.sup.type_ = min(existing.sup.type_, props.sup.type_);
                            } else {
                                spellings.insert(item.syllable_id, props);
                            }
                        }
                    }

                    if spellings.is_empty() {
                        info!("Not spelled");
                        end_vertices.remove(&end_pos);
                        continue;
                    }
                    // Find the best common type in a path up to the end vertex
                    // e.g. pinyin "shurfa" has vertex type Normal at position 3,
                    // Abbreviation at position 4 and Abbreviation at position 6
                    if end_vertex_type < current_type {
                        end_vertex_type = current_type;
                    }
                    queue.push((end_pos, end_vertex_type));
                    info!(
                        "Added to syllable graph, edge: [{}, {})",
                        current_pos, end_pos
                    );
                }
            }
        }

        info!("Remove stale vertices and edges");
        let mut good = HashSet::new();
        good.insert(farthest);
        // Fuzzy spellings are immune to invalidation by normal spellings
        let last_type = max(graph.vertices[&farthest], SpellingType::Fuzzy);
        let mut overlaps_to_check = Vec::new();
        for i in (0..farthest).rev() {
            if !graph.vertices.contains_key(&i) {
                continue;
            }
            // Remove stale edges
            if let Some(map) = &mut graph.edges.get_mut(&i) {
                map.retain(|key, value| {
                    if !good.contains(key) {
                        // Not connected
                        return false;
                    }
                    // Remove disqualified syllables (e.g. matching abbreviated spellings)
                    // when there is a path of more favored type
                    let mut edge_type = SpellingType::Invalid;
                    value.retain(|_, props| {
                        if props.is_correction {
                            return true; // Don't care correction edges
                        }
                        if props.sup.type_ > last_type {
                            return false;
                        } else {
                            edge_type = min(edge_type, props.sup.type_);
                            return true;
                        }
                    });

                    if !value.is_empty() {
                        if edge_type < SpellingType::Abbreviation {
                            overlaps_to_check.push((i, *key));
                        }
                        return true;
                    }
                    false
                });
            }
            if graph.vertices[&i] > last_type || graph.edges[&i].is_empty() {
                info!("Remove stale vertex at {}", i);
                graph.vertices.remove(&i);
                graph.edges.remove(&i);
                continue;
            }
            // Keep the valid vertex
            good.insert(i);
        }

        for (start, end) in overlaps_to_check {
            self.check_overlapped_spellings(graph, start, end);
        }

        if self.enable_completion && farthest < input.len() {
            info!("Completion enabled");
            let expand_search_limit = 512;
            let mut keys = Vec::new();
            prism.expand_search(&input[farthest..], &mut keys, expand_search_limit);
            if !keys.is_empty() {
                let current_pos = farthest;
                let end_pos = input.len();
                let code_length = end_pos - current_pos;
                let end_vertices = graph.edges.entry(current_pos).or_default();
                let spellings = end_vertices.entry(end_pos).or_default();
                for m in keys {
                    if m.length < code_length {
                        continue;
                    }
                    // When spelling algebra is enabled,
                    // a spelling evaluates to a set of syllables;
                    // otherwise, it resembles exactly the syllable itself.
                    let accessor = prism.query_spelling(m.value);
                    if let Some(list) = accessor.spelling_map.get(m.value as usize) {
                        for item in list {
                            let mut props = EdgeProperties::new(accessor.properties());
                            if props.sup.type_ < SpellingType::Abbreviation {
                                props.sup.type_ = SpellingType::Completion;
                                props.sup.credibility += COMPLETION_PENALTY;
                                props.sup.end_pos = end_pos;
                                // Add a syllable with properties to the edge's
                                // spelling-to-syllable map
                                spellings.insert(item.syllable_id, props);
                            }
                        }
                    }
                }
                if spellings.is_empty() {
                    info!("No completion could be made");
                    end_vertices.remove(&end_pos);
                } else {
                    info!(
                        "Added to syllable graph, completion: [{}, {})",
                        current_pos, end_pos
                    );
                    farthest = end_pos;
                }
            }
        }

        graph.input_length = input.len();
        graph.interpreted_length = farthest;
        info!("Input length: {}", graph.input_length);
        info!("Syllabified length: {}", graph.interpreted_length);
        self.transpose(graph);

        farthest
    }

    fn check_overlapped_spellings(&self, graph: &mut SyllableGraph, start: usize, end: usize) {
        if !graph.edges.contains_key(&start) {
            return;
        }

        let mut joints_to_update = Vec::new();
        // If "Z" = "YX", mark the vertex between Y and X an ambiguous syllable joint
        {
            let y_end_vertices = &graph.edges[&start];
            // Enumerate Ys
            for &joint in y_end_vertices.keys() {
                if joint >= end {
                    break;
                }
                // Test X
                if !graph.edges.contains_key(&joint) {
                    continue;
                }
                let x_end_vertices = &graph.edges[&joint];
                for &key in x_end_vertices.keys() {
                    if key < end {
                        continue;
                    }
                    if key == end {
                        // Discourage syllables at an ambiguous joint.
                        // Bad cases include pinyin syllabification "niju'ede".
                        joints_to_update.push(joint);
                    }
                    break;
                }
            }
        }

        for joint in joints_to_update {
            if let Some(x_end_vertices) = graph.edges.get_mut(&joint) {
                for (key, value) in x_end_vertices.iter_mut() {
                    if *key == end {
                        for (_, props) in value.iter_mut() {
                            props.sup.credibility += PENALTY_FOR_AMBIGUOUS_SYLLABLE;
                        }
                        graph.vertices.insert(joint, SpellingType::Ambiguous);
                        info!("Ambiguous syllable joint at position {}", joint);
                    }
                }
            }
        }
    }

    fn transpose(&self, graph: &mut SyllableGraph) {
        for (&start, value) in &graph.edges {
            let index = graph.indices.entry(start).or_default();
            for (_, spelling_map) in value.iter().rev() {
                for (syll_id, spelling) in spelling_map {
                    index.entry(*syll_id).or_default().push(spelling);
                }
            }
        }
    }

    fn enable_correction(&mut self, corrector: &mut Corrector) {
        self.corrector = Some(corrector);
    }
}

fn main() {
    // Example usage
    let delimiters = String::from(" ,.;");
    let syllabifier = Syllabifier::new(delimiters, true, true);
    let input = String::from("example input");
    let mut prism = Prism;
    let mut graph = SyllableGraph::default();

    syllabifier.build_syllable_graph(&input, &mut prism, &mut graph);

    println!("Input length: {}", graph.input_length);
    println!("Interpreted length: {}", graph.interpreted_length);
}
