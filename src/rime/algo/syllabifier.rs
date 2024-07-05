use std::cmp::{max, min, Reverse};
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashSet};
use std::ops::{Deref, DerefMut};

use log::info;

use crate::rime::algo::spelling::{SpellingProperties, SpellingType};
use crate::rime::algo::SyllableId;
use crate::rime::dict::corrector::{Corrections, Corrector};
use crate::rime::dict::prism::{Match, Prism, SpellingDescriptor};

const COMPLETION_PENALTY: f64 = -std::f64::consts::LN_2;
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

    fn _peek(&self) -> Option<&Vertex> {
        self.0.peek().map(|Reverse(item)| item)
    }
}

#[derive(Clone, Debug, Default)]
pub struct EdgeProperties {
    sup: SpellingProperties,
    is_correction: bool,
}

impl EdgeProperties {
    fn new(sup: SpellingProperties) -> Self {
        Self {
            sup,
            is_correction: false,
        }
    }

    pub fn is_correction(&self) -> bool {
        self.is_correction
    }
}

impl Deref for EdgeProperties {
    type Target = SpellingProperties;

    fn deref(&self) -> &Self::Target {
        &self.sup
    }
}

impl DerefMut for EdgeProperties {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sup
    }
}

type VertexMap = BTreeMap<usize, SpellingType>;
type SpellingMap = BTreeMap<SyllableId, EdgeProperties>;
type EndVertexMap = BTreeMap<usize, SpellingMap>;
type EdgeMap = BTreeMap<usize, EndVertexMap>;

type SpellingPropertiesList = Vec<EdgeProperties>;
type SpellingIndex = BTreeMap<SyllableId, SpellingPropertiesList>;
type SpellingIndices = BTreeMap<usize, SpellingIndex>;

#[derive(Default, Debug)]
pub struct SyllableGraph {
    input_length: usize,
    interpreted_length: usize,
    vertices: VertexMap,
    edges: EdgeMap,
    indices: SpellingIndices,
}

impl SyllableGraph {
    pub fn input_length(&self) -> usize {
        self.input_length
    }

    pub fn interpreted_length(&self) -> usize {
        self.interpreted_length
    }

    pub fn vertices(&self) -> &VertexMap {
        &self.vertices
    }

    pub fn edges(&self) -> &EdgeMap {
        &self.edges
    }

    pub fn indices(&self) -> &SpellingIndices {
        &self.indices
    }
}

#[derive(Default)]
pub struct Syllabifier<'a, T: Corrector> {
    delimiters: String,
    enable_completion: bool,
    strict_spelling: bool,
    corrector: Option<&'a mut T>,
}

impl<'a, T: Corrector> Syllabifier<'a, T> {
    fn _new(delimiters: String, enable_completion: bool, strict_spelling: bool) -> Self {
        Self {
            delimiters,
            enable_completion,
            strict_spelling,
            corrector: None,
        }
    }

    pub fn build_syllable_graph(
        &self,
        input: &str,
        prism: &Prism,
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
            let mut exact_match_syllables = BTreeSet::new();
            let mut matches = prism
                .common_prefix_search(current_input)
                .unwrap_or_default();

            if let Some(ref corrector) = self.corrector {
                exact_match_syllables = matches.iter().map(|matched| matched.value).collect();

                let mut corrections = Corrections::new();
                corrector.tolerance_search(prism, current_input, &mut corrections, 5);
                for (&syllable_id, correction) in corrections.deref() {
                    let list = match prism.query_spelling(syllable_id as usize) {
                        Some(list) => list,
                        None => &vec![SpellingDescriptor::new(syllable_id)],
                    };

                    for item in list {
                        if item.type_ as usize == SpellingType::Normal as usize {
                            matches.push((correction.offset, syllable_id).into());
                            break;
                        }
                    }
                }
            }

            if !matches.is_empty() {
                let end_vertices = graph.edges.entry(current_pos).or_default();
                for Match { offset, value } in &matches {
                    if *offset == 0 {
                        continue;
                    }
                    let mut end_pos = current_pos + offset;
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

                    // When spelling algebra is enabled, a spelling evaluates to a set of syllables;
                    // otherwise, it resembles exactly the syllable itself.
                    let list = match prism.query_spelling(*value as usize) {
                        Some(list) => list,
                        None => &vec![SpellingDescriptor::new(*value as SyllableId)],
                    };

                    for item in list {
                        let mut props = EdgeProperties::new(item.properties());
                        if self.strict_spelling
                            && matches_input
                            && props.type_ != SpellingType::Normal
                        {
                            // Disqualify fuzzy spelling or abbreviation as single word
                            continue;
                        }

                        props.end_pos = end_pos;

                        // Add a syllable with properties to the edge's spelling-to-syllable map
                        if self.corrector.is_some() && !exact_match_syllables.contains(&value) {
                            props.is_correction = true;
                            props.credibility = CORRECTION_CREDIBILITY;
                        }

                        // Let end_vertex_type be the best (smaller) type of spelling that ends at the vertex
                        if end_vertex_type > props.type_ && !props.is_correction {
                            end_vertex_type = props.type_;
                        }

                        if let Some(existing) = spellings.get_mut(&item.syllable_id) {
                            existing.type_ = min(existing.type_, props.type_);
                        } else {
                            spellings.insert(item.syllable_id, props);
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
                        return if props.type_ > last_type {
                            false
                        } else {
                            edge_type = min(edge_type, props.type_);
                            true
                        };
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

            if graph.vertices[&i] > last_type || graph.edges.get(&i).map_or(true, |e| e.is_empty())
            {
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
            let keys = prism
                .expand_search(&input[farthest..], expand_search_limit)
                .unwrap_or_default();
            if !keys.is_empty() {
                let current_pos = farthest;
                let end_pos = input.len();
                let code_length = end_pos - current_pos;
                let end_vertices = graph.edges.entry(current_pos).or_default();
                let spellings = end_vertices.entry(end_pos).or_default();
                for Match { offset, value } in keys {
                    if offset < code_length {
                        continue;
                    }
                    // When spelling algebra is enabled,
                    // a spelling evaluates to a set of syllables;
                    // otherwise, it resembles exactly the syllable itself.
                    if let Some(list) = prism.query_spelling(value as usize) {
                        for item in list {
                            let mut props = EdgeProperties::new(item.properties());
                            if props.type_ < SpellingType::Abbreviation {
                                props.type_ = SpellingType::Completion;
                                props.credibility += COMPLETION_PENALTY;
                                props.end_pos = end_pos;
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
                            props.credibility += PENALTY_FOR_AMBIGUOUS_SYLLABLE;
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
                    index.entry(*syll_id).or_default().push(spelling.clone());
                }
            }
        }
    }

    pub fn enable_correction(&mut self, corrector: &'a mut T) {
        self.corrector = Some(corrector);
    }
}
