//use crate::rime::common::PathExt;

use crate::rime::algo::SyllableId;
use crate::rime::common::PathExt;

#[derive(Clone, Debug)]
pub struct StringType {
    value: i32,
}

impl StringType {
    pub fn str(&self) -> String {
        // 模拟获取字符串内容，实际实现需要从 `StringTable` 中提取
        format!("str_id_{}", self.value)
    }
}

type Syllabary = Vec<StringType>;

pub type Code = Vec<SyllableId>;

pub type Weight = f32;

#[derive(Clone, Debug)]
pub struct Entry {
    pub text: StringType,
    pub weight: Weight,
}

#[derive(Clone, Debug)]
pub struct LongEntry {
    pub extra_code: Code,
    pub entry: Entry,
}

#[derive(Clone, Debug)]
pub struct HeadIndexNode {
    pub entries: Vec<Entry>,
    pub next_level: Option<PhraseIndex>,
}

pub type HeadIndex = Vec<HeadIndexNode>;

#[derive(Clone, Debug)]
pub struct TrunkIndexNode {
    pub key: SyllableId,
    pub entries: Vec<Entry>,
    pub next_level: Option<PhraseIndex>,
}

pub type TrunkIndex = Vec<TrunkIndexNode>;
pub type TailIndex = Vec<LongEntry>;

#[derive(Clone, Debug)]
pub enum PhraseIndex {
    Trunk(TrunkIndex),
    Tail(TailIndex),
}

pub type Index = HeadIndex;

pub struct Table {
    file_path: PathExt,
    syllabary: Syllabary,
    //vocabulary: Vocabulary,
}

//type Vocabulary = HashMap<usize, Vec<ShortDictEntry>>;

impl Table {
    pub fn new(file_path: PathExt) -> Self {
        // 省略文件加载逻辑，假设构造成功
        Self {
            file_path,
            syllabary: Vec::new(),
        }
    }

    //    fn remove(&mut self) {
    //        // 清空内容
    //        self.syllabary.clear();
    //        self.vocabulary.clear();
    //    }

    //    fn build(&mut self, syll: &Syllabary, voc: &Vocabulary, total_num_entries: usize) -> bool {
    //        self.syllabary = syll.clone();
    //        self.vocabulary = voc.clone();
    //        true // 假设构建成功
    //    }

    //    fn save(&self) -> bool {
    //        // 假设保存成功
    //        true
    //    }

    //    fn load(&mut self) -> bool {
    //        // 假设加载成功
    //        true
    //    }

    //    fn close(&mut self) {
    //        // 关闭文件等操作
    //    }

    //    fn get_syllable_by_id(&self, id: usize) -> Option<&str> {
    //        self.syllabary
    //            .iter()
    //            .find_map(|(k, &v)| if v == id { Some(k.as_str()) } else { None })
    //    }

    //    fn query_words(&self, code: usize) -> Option<&Vec<ShortDictEntry>> {
    //        self.vocabulary.get(&code)
    //    }

    //    fn query_phrases(&self, code: &[usize]) -> Option<Vec<&ShortDictEntry>> {
    //        let mut result = Vec::new();
    //        for c in code {
    //            if let Some(entries) = self.vocabulary.get(c) {
    //                result.extend(entries.iter());
    //            }
    //        }
    //        Some(result)
    //    }
}
