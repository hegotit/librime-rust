use crate::rime::algo::{spelling::SpellingProperties, SyllableId};

pub(crate) struct SpellingDescriptor {
    pub(crate) syllable_id: SyllableId,
    pub(crate) type_: i32,
    credibility: f32,
    tips: String,
}

type SpellingMapItem = Vec<SpellingDescriptor>;
type SpellingMap = Vec<SpellingMapItem>;

pub(crate) struct SpellingAccessor {
    syllable_id: SyllableId,
    pub(crate) spelling_map: SpellingMap,
}

impl SpellingAccessor {
    pub(crate) fn syllable_id(&self) -> SyllableId {
        todo!()
    }
    pub(crate) fn properties(&self) -> SpellingProperties {
        todo!()
    }
}

pub(crate) struct Prism;

impl Prism {
    pub(crate) fn common_prefix_search(&self, _input: &str, _matches: &mut Vec<Match>) {
        todo!()
    }
    pub(crate) fn query_spelling(&self, _value: SyllableId) -> SpellingAccessor {
        todo!()
    }
    pub(crate) fn expand_search(&self, _input: &str, _keys: &mut Vec<Match>, _limit: usize) {}
}

pub(crate) struct Match {
    pub(crate) value: SyllableId,
    pub(crate) length: usize,
}
