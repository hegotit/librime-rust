// marisa-rs crate can be used for marisa trie in Rust
// Add dependencies in your Cargo.toml file:
// marisa = "0.1"

//use marisa::{Agent, Keyset, Trie};
use std::collections::{HashMap, VecDeque};

use trie_rs::{Trie, TrieBuilder};

pub const INVALID_STRING_ID: u32 = u32::MAX;

pub struct StringTable {
    trie: Trie<u8>,
    id_to_string: HashMap<i32, String>,
}

impl StringTable {
    //pub fn new() -> Self {
    //    Self {
    //        trie: TrieBuilder::new().build(),
    //        id_to_string: HashMap::new(),
    //    }
    //}

    //pub fn has_key(&self, key: &str) -> bool {
    //    self.trie.exact_match(key.as_bytes())
    //}

    //pub fn lookup(&self, key: &str) -> StringId {
    //    //let mut agent = Agent::new();
    //    //agent.set_query(key);
    //    //if self.trie.lookup(&mut agent) {
    //    //    agent.key().id() as StringId
    //    //} else {
    //    //    INVALID_STRING_ID
    //    //}

    //    if let Some(_) = self.trie.exact_match(key.as_bytes()) {
    //        Some(self.trie.exact_match(key.as_bytes()).unwrap() as StringId)
    //    } else {
    //        None
    //    }
    //}

    //pub fn common_prefix_match(&self, query: &str) -> Vec<StringId> {
    //    let mut agent = Agent::new();
    //    agent.set_query(query);
    //    let mut result = Vec::new();
    //    while self.trie.common_prefix_search(&mut agent) {
    //        result.push(agent.key().id() as StringId);
    //    }
    //    result
    //}

    //pub fn predict(&self, query: &str) -> Vec<StringId> {
    //    let mut agent = Agent::new();
    //    agent.set_query(query);
    //    let mut result = Vec::new();
    //    while self.trie.predictive_search(&mut agent) {
    //        result.push(agent.key().id() as StringId);
    //    }
    //    result
    //}

    //pub fn get_string(&self, string_id: StringId) -> Option<String> {
    //    let mut agent = Agent::new();
    //    agent.set_query(string_id as usize);
    //    if self.trie.reverse_lookup(&mut agent).is_ok() {
    //        Some(agent.key().to_str().unwrap().to_string())
    //    } else {
    //        None
    //    }
    //}

    //pub fn num_keys(&self) -> usize {
    //    self.trie.num_keys()
    //}

    //pub fn binary_size(&self) -> usize {
    //    self.trie.io_size()
    //}
}

//pub struct StringTableBuilder {
//    keys: Keyset,
//    references: VecDeque<Option<StringId>>,
//}

//impl StringTableBuilder {
//    pub fn new() -> Self {
//        Self {
//            keys: Keyset::new(),
//            references: VecDeque::new(),
//        }
//    }

//    pub fn add(&mut self, key: &str, weight: f64, reference: Option<&mut StringId>) {
//        self.keys.push(key, weight as f32);
//        if let Some(ref mut ref_value) = reference {
//            self.references.push_back(Some(*ref_value));
//        } else {
//            self.references.push_back(None);
//        }
//    }

//    pub fn clear(&mut self) {
//        self.keys.clear();
//        self.references.clear();
//    }

//    pub fn build(&mut self) -> Trie {
//        let trie = Trie::build(&self.keys);
//        self.update_references(&trie);
//        trie
//    }

//    fn update_references(&mut self, trie: &Trie) {
//        let mut agent = Agent::new();
//        for i in 0..self.keys.len() {
//            if let Some(ref_value) = self.references.pop_front() {
//                if let Some(mut ref_value) = ref_value {
//                    agent.set_query(self.keys[i].as_str());
//                    if trie.lookup(&mut agent) {
//                        ref_value = agent.key().id() as StringId;
//                    }
//                }
//            }
//        }
//    }

//    pub fn dump(&self, data: &mut [u8]) {
//        let size_needed = self.keys.io_size();
//        if data.len() < size_needed {
//            eprintln!("Insufficient memory to dump string table.");
//            return;
//        }
//        self.keys.write(&mut data[..size_needed]);
//    }
//}
