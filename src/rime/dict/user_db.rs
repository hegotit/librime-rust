use log::{error, info};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::{Arc, LazyLock};

use crate::rime::common::PathExt;
use crate::rime::dict::db::Db;
use crate::rime::dict::db_utils::{DbSink, DbSource};
use crate::rime::dict::tsv::{TsvFormatter, TsvParser, TsvReader, TsvWriter};
use crate::rime::service::Service;

type TickCount = u64;

static PLAIN_USERDB_EXTENSION: &str = ".userdb.txt";

pub(crate) static PLAIN_USERDB_FORMAT: LazyLock<TextFormat> = LazyLock::new(|| {
    TextFormat::new(
        Arc::new(userdb_entry_parser),
        Arc::new(userdb_entry_formatter),
        "Rime user dictionary".to_string(),
    )
});

pub(crate) struct TextFormat {
    pub(crate) parser: TsvParser,
    pub(crate) formatter: TsvFormatter,
    pub(crate) file_description: String,
}

impl TextFormat {
    pub(crate) fn new(
        parser: TsvParser,
        formatter: TsvFormatter,
        file_description: String,
    ) -> Self {
        Self {
            parser,
            formatter,
            file_description,
        }
    }
}

impl std::fmt::Debug for TextFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextFormat")
            .field("parser", &"TsvParser")
            .field("formatter", &"TsvFormatter")
            .field("file_description", &self.file_description)
            .finish()
    }
}

/// Properties of a user db entry value.
#[derive(Default)]
struct UserDbValue {
    commits: i32,
    dee: f64,
    tick: TickCount,
}

impl UserDbValue {
    fn new() -> Self {
        Default::default()
    }
}

impl FromStr for UserDbValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut entry = Self::new();
        entry.unpack(s);
        Ok(entry)
    }
}

impl UserDbValue {
    fn pack(&self) -> String {
        format!("c={} d={} t={}", self.commits, self.dee, self.tick)
    }

    fn unpack(&mut self, value: &str) -> bool {
        let kv_pairs: Vec<&str> = value.split_whitespace().collect();
        for kv in kv_pairs {
            let parts: Vec<&str> = kv.split('=').collect();
            if parts.len() != 2 {
                continue;
            }

            let k = parts[0];
            let v = parts[1];
            match k {
                "c" => {
                    self.commits = i32::from_str(v).unwrap_or_default();
                }
                "d" => {
                    self.dee = f64::from_str(v).unwrap_or_default().min(10000.0);
                }
                "t" => {
                    self.tick = u64::from_str(v).unwrap_or_default();
                }
                _ => {
                    error!("Failed in parsing key-value from userdb entry '{}'.", kv);
                    return false;
                }
            }
        }
        true
    }
}

/**
 * A placeholder struct for user db.
 *
 * Note: do not directly use this struct to instantiate a user db.
 * Instead, use the rime::UserDbWrapper<T> template, which creates
 * wrapper classes for underlying implementations of rime::Db.
 */
struct UserDb {
    // 示例字段
}

impl UserDb {
    fn snapshot_extension() -> &'static str {
        PLAIN_USERDB_EXTENSION
    }
}

/// A helper to provide extra functionalities related to user db.
pub(crate) struct UserDbHelper<'a, T>
where
    T: Db,
{
    db: &'a mut T,
}

impl<'a, T> UserDbHelper<'a, T>
where
    T: Db,
{
    pub(crate) fn new(db: &'a mut T) -> Self {
        Self { db }
    }

    pub(crate) fn update_user_info(&'a mut self) -> bool {
        let user_id = &Service::instance().deployer().user_id;
        self.db
            .meta_update("/user_id".to_owned(), user_id.to_owned())
    }

    fn is_uniform_format(file_path: &PathExt) -> bool {
        if let Some(filename) = file_path.to_str() {
            filename.ends_with(UserDb::snapshot_extension())
        } else {
            false
        }
    }

    fn uniform_backup(&mut self, snapshot_file: &PathExt) -> bool {
        info!(
            "Backing up userdb '{}' to {:?}",
            self.db.name(),
            snapshot_file
        );

        let mut writer =
            TsvWriter::new(snapshot_file.clone(), PLAIN_USERDB_FORMAT.formatter.clone());
        writer.file_description = PLAIN_USERDB_FORMAT.file_description.clone();

        //let metadata: Option<Arc<TextDbAccessor<'_>>> = self.db.query_metadata::<TextDbAccessor>();
        //let data: Option<Arc<TextDbAccessor<'_>>> = self.db.query_all::<TextDbAccessor>();
        let mut source = DbSource::new(self.db);

        match writer << &mut source {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }

    fn uniform_restore(&mut self, snapshot_file: &PathExt) -> bool {
        info!(
            "Restoring userdb '{}' from {:?}",
            self.db.name(),
            snapshot_file
        );
        let reader = {
            let parser: TsvParser = PLAIN_USERDB_FORMAT.parser.clone();
            TsvReader::new(snapshot_file.clone(), parser)
        };

        let mut sink = DbSink::new(self.db);

        match reader >> &mut sink {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }

    fn is_user_db(&self) -> bool {
        self.db
            .meta_fetch("/db_type")
            .map_or(false, |db_type| db_type == "userdb")
    }

    fn get_db_name(&self) -> String {
        self.db
            .meta_fetch("/db_name")
            .map(|mut name| {
                if let Some(ext_index) = name.rfind(".userdb") {
                    name.truncate(ext_index);
                }
                name
            })
            .unwrap_or_default()
    }

    fn get_user_id(&self) -> String {
        self.db
            .meta_fetch("/user_id")
            .unwrap_or_else(|| String::from("unknown"))
    }

    fn get_rime_version(&self) -> String {
        self.db.meta_fetch("/rime_version").unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct UserDbWrapper<T>
where
    T: Db,
{
    pub db: T,
}

impl<T> Deref for UserDbWrapper<T>
where
    T: Db,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<T> DerefMut for UserDbWrapper<T>
where
    T: Db,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

impl<T> UserDbWrapper<T>
where
    T: Db,
{
    pub fn new(file_path: PathExt, db_name: &str) -> Self {
        Self {
            db: T::build(file_path, db_name),
        }
    }

    pub fn open(&mut self) -> bool {
        self.db.open(true)
    }

    //fn backup(&mut self, snapshot_file: &PathExt) -> bool {
    //    if UserDbHelper::<T>::is_uniform_format(snapshot_file) {
    //        UserDbHelper::new(&mut self.0).uniform_backup(snapshot_file)
    //    } else {
    //        self.0.backup(snapshot_file)
    //    }
    //}

    //fn restore(&mut self, snapshot_file: &PathExt) -> bool {
    //    if UserDbHelper::<T>::is_uniform_format(snapshot_file) {
    //        UserDbHelper::new(&mut self.0).uniform_restore(snapshot_file)
    //    } else {
    //        self.0.restore(snapshot_file)
    //    }
    //}
    //}

    //impl<'a, T> UserDbWrapper<TextDb<'a, T>>
    //where
    //    T: BaseDb,
    //{
    //    pub fn new(file_path: PathExt, db_name: &str) -> Self {
    //        let db: Db = Db::new(file_path, db_name);
    //        Self {
    //            0: TextDb::new(db, "userdb", PLAIN_USERDB_FORMAT.deref()),
    //        }
    //        //todo!()
    //    }
    //}

    //struct UserDbComponent<T: Db> {
    //    extension: &'static str,
    //    phantom: std::marker::PhantomData<T>,
    //}

    //impl<T> UserDbComponent<T>
    //where
    //    T: Db,
    //{
    //    fn create(name: &str) -> T {
    //        //Box::new(UserDbWrapper::<T>::new(&Path::new(name), name))
    //        todo!()
    //    }

    //    fn extension(&self) -> &'static str {
    //        self.extension
    //    }
    //}

    ///// 用户数据库合并类
    //struct UserDbMerger<'a> {
    //    db: &'a mut dyn Db,
    //    our_tick: TickCount,
    //    their_tick: TickCount,
    //    max_tick: TickCount,
    //    merged_entries: i32,
    //}

    //impl<'a> UserDbMerger<'a> {
    //    fn new(db: &'a mut dyn Db) -> Self {
    //        Self {
    //            db,
    //            our_tick: get_tick_count(db),
    //            their_tick: 0,
    //            max_tick: get_tick_count(db),
    //            merged_entries: 0,
    //        }
    //    }

    //    fn meta_put(&mut self, key: &str, value: &str) -> bool {
    //        if key == "/tick" {
    //            self.their_tick = u64::from_str(value).unwrap_or(0);
    //            self.max_tick = max(self.our_tick, self.their_tick);
    //        }
    //        true
    //    }

    //    fn put(&mut self, key: &str, value: &str) -> bool {
    //        if self.db.is_none() {
    //            return false;
    //        }
    //        let mut v = UserDbValue::from_string(value);
    //        if v.tick < self.their_tick {
    //            v.dee = algo::formula_d(0.0, self.their_tick as f64, v.dee, v.tick as f64);
    //        }
    //        let mut o = UserDbValue::new();
    //        if let Ok(our_value) = self.db.fetch(key) {
    //            o.unpack(&our_value);
    //        }
    //        if o.tick < self.our_tick {
    //            o.dee = algo::formula_d(0.0, self.our_tick as f64, o.dee, o.tick as f64);
    //        }
    //        if o.commits.abs() < v.commits.abs() {
    //            o.commits = v.commits;
    //        }
    //        o.dee = max(o.dee, v.dee);
    //        o.tick = self.max_tick;
    //        self.db.update(key, &o.pack()) && {
    //            self.merged_entries += 1;
    //            true
    //        }
    //    }

    //    fn close_merge(&mut self) {
    //        if self.db.is_none() || self.merged_entries == 0 {
    //            return;
    //        }
    //        let deployer = Service::instance().deployer();
    //        if self
    //            .db
    //            .meta_update("/tick", &self.max_tick.to_string())
    //            .is_err()
    //        {
    //            error!("failed to update tick count.");
    //            return;
    //        }
    //        if self
    //            .db
    //            .meta_update("/user_id", &deployer.user_id())
    //            .is_err()
    //        {
    //            error!("failed to update user_id.");
    //            return;
    //        }
    //        info!(
    //            "total {} entries merged, tick = {}",
    //            self.merged_entries, self.max_tick
    //        );
    //        self.merged_entries = 0;
    //    }
    //}

    ///// 用户数据库导入类
    //struct UserDbImporter<'a> {
    //    db: &'a mut dyn Db,
    //}

    //impl<'a> UserDbImporter<'a> {
    //    fn new(db: &'a mut dyn Db) -> Self {
    //        Self { db }
    //    }

    //    fn meta_put(&self, _key: &str, _value: &str) -> bool {
    //        true
    //    }

    //    fn put(&mut self, key: &str, value: &str) -> bool {
    //        if self.db.is_none() {
    //            return false;
    //        }
    //        let v = UserDbValue::from_string(value);
    //        let mut o = UserDbValue::new();
    //        if let Ok(old_value) = self.db.fetch(key) {
    //            o.unpack(&old_value);

    //            if v.commits > 0 {
    //                o.commits = max(o.commits, v.commits);
    //                o.dee = max(o.dee, v.dee);
    //            } else if v.commits < 0 {
    //                // 标记为已删除
    //                o.commits = min(v.commits, -o.commits.abs());
    //            }
    //            self.db.update(key, &o.pack())
    //        }
    //    }

    //    /// 获取数据库的tick计数
    //    fn get_tick_count(db: &dyn Db) -> TickCount {
    //        if let Ok(tick) = db.meta_fetch("/tick") {
    //            if let Ok(tick_value) = u64::from_str(&tick) {
    //                return tick_value;
    //            }
    //        }
    //        1
    //    }
}

fn userdb_entry_parser(row: &[&str]) -> Option<(String, String)> {
    match row {
        [code, value, rest @ ..] if !code.is_empty() && !value.is_empty() => {
            let key = if code.ends_with(' ') {
                format!("{}\t{}", code, value)
            } else {
                // Add a space to invalid keys generated by a previous buggy version
                format!("{} \t{}", code, value)
            };

            let val = rest.first().map_or_else(String::new, |v| v.to_string());
            Some((key, val))
        }
        _ => None,
    }
}

fn userdb_entry_formatter(key: &str, value: &str) -> Option<Vec<String>> {
    let mut parts = key.split('\t');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(k), Some(v), None) if !k.is_empty() && !v.is_empty() => {
            Some(vec![k.to_string(), v.to_string(), value.to_string()])
        }
        _ => None,
    }
}
