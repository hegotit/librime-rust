////use crate::rime::common::PathExt;

//use log::info;

//use crate::rime::algo::SyllableId;
//use crate::rime::common::PathExt;
//use crate::rime::dict::vocabulary::{Syllabary as SyllSet, Vocabulary};

//#[derive(Clone, Debug)]
//pub struct StringType {
//    value: i32,
//}

//impl StringType {
//    pub fn str(&self) -> String {
//        self.value.to_string()
//    }

//    pub fn str_id(&self) -> u32 {
//        self.value as u32
//    }
//}

//type Syllabary = Vec<StringType>;

//pub type Code = Vec<SyllableId>;

//pub type Weight = f32;

//#[derive(Clone, Debug)]
//pub struct Entry {
//    pub text: StringType,
//    pub weight: Weight,
//}

//#[derive(Clone, Debug)]
//pub struct LongEntry {
//    pub extra_code: Code,
//    pub entry: Entry,
//}

//#[derive(Clone, Debug)]
//pub struct HeadIndexNode {
//    pub entries: Vec<Entry>,
//    pub next_level: Option<PhraseIndex>,
//}

//pub type HeadIndex = Vec<HeadIndexNode>;

//#[derive(Clone, Debug)]
//pub struct TrunkIndexNode {
//    pub key: SyllableId,
//    pub entries: Vec<Entry>,
//    pub next_level: Option<PhraseIndex>,
//}

//pub type TrunkIndex = Vec<TrunkIndexNode>;
//pub type TailIndex = Vec<LongEntry>;

//#[derive(Clone, Debug)]
//pub enum PhraseIndex {
//    Trunk(TrunkIndex),
//    Tail(TailIndex),
//}

//pub type Index = HeadIndex;

//struct Metadata {
//    format: String,
//    dict_file_checksum: u32,
//    num_syllables: u32,
//    num_entries: u32,
//    string_table_size: u32,
//}

//pub struct Table {
//    file_path: PathExt,
//    metadata: Option<Metadata>,
//    syllabary: Option<Syllabary>,
//    string_table_builder : Option<>
//}

////type Vocabulary = HashMap<usize, Vec<ShortDictEntry>>;

//impl Table {
//    pub fn new(file_path: PathExt) -> Self {
//        Self {
//            file_path,
//            metadata: None,
//            syllabary: None,
//        }
//    }

//    //    fn remove(&mut self) {
//    //        // 清空内容
//    //        self.syllabary.clear();
//    //        self.vocabulary.clear();
//    //    }

//    pub fn build(
//        &mut self,
//        syllabary: &SyllSet,
//        vocabulary: &Vocabulary,
//        num_entries: usize,
//        dict_file_checksum: u32,
//    ) -> bool {
//        let num_syllables = syllabary.len();
//        //let estimated_file_size = K_RESERVED_SIZE + 32 * num_syllables + 64 * num_entries;

//        info!("building table.");
//        info!("num syllables: {}", num_syllables);
//        info!("num entries: {}", num_entries);

//        info!("creating metadata.");

//        self.metadata = Some(Metadata {
//            format: String::new(),
//            dict_file_checksum,
//            num_syllables: num_syllables as u32,
//            num_entries: num_entries as u32,
//            string_table_size: 0,
//        });

//        self.string_table_builder_ = StringTableBuilder::default();

//        //println!("creating syllabary.");
//        //self.syllabary = Some(syllabary.clone());
//        //if let Some(metadata) = &mut self.metadata {
//        //    metadata.syllabary = Some(syllabary.clone());
//        //}

//        //println!("creating table index.");
//        //self.index = Some(self.build_index(vocabulary, num_syllables));
//        //if let Some(metadata) = &mut self.metadata {
//        //    metadata.index = self.index.clone();
//        //}

//        //if !self.on_build_finish() {
//        //    return false;
//        //}

//        //if let Some(metadata) = &mut self.metadata {
//        //    metadata.format = K_TABLE_FORMAT_LATEST.to_string();
//        //}

//        true // 假设构建成功
//    }

//    //    fn save(&self) -> bool {
//    //        // 假设保存成功
//    //        true
//    //    }

//    //    fn load(&mut self) -> bool {
//    //        // 假设加载成功
//    //        true
//    //    }

//    //    fn close(&mut self) {
//    //        // 关闭文件等操作
//    //    }

//    fn get_syllable_by_id(&self, syllable_id: SyllableId) -> Option<&str> {
//        if let Some(syllabary) = &self.syllabary {
//            if syllable_id < syllabary.len() as SyllableId {
//                return Some(syllabary[syllable_id as usize].str_id());
//            }
//        }
//        None
//    }

//    //    fn query_words(&self, code: usize) -> Option<&Vec<ShortDictEntry>> {
//    //        self.vocabulary.get(&code)
//    //    }

//    //    fn query_phrases(&self, code: &[usize]) -> Option<Vec<&ShortDictEntry>> {
//    //        let mut result = Vec::new();
//    //        for c in code {
//    //            if let Some(entries) = self.vocabulary.get(c) {
//    //                result.extend(entries.iter());
//    //            }
//    //        }
//    //        Some(result)
//    //    }
//}
