use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

type RimeSessionId = usize;
type Bool = c_int;
const FALSE: Bool = 0;
const TRUE: Bool = 1;
type RimeProtoBuilder = c_void;

#[repr(C)]
pub struct RimeTraits {
    data_size: c_int,
    shared_data_dir: *const c_char,
    user_data_dir: *const c_char,
    distribution_name: *const c_char,
    distribution_code_name: *const c_char,
    distribution_version: *const c_char,
    /// Pass a C-string constant in the format "rime.x"
    /// where 'x' is the name of your application.
    /// Add prefix "rime." to ensure old log files are automatically cleaned.
    app_name: *const c_char,
    /// A list of modules to load before initializing
    modules: *const *const c_char,
    /// Minimal level of logged messages.
    /// Value is passed to Glog library using FLAGS_minloglevel variable.
    /// 0 = INFO (default), 1 = WARNING, 2 = ERROR, 3 = FATAL
    min_log_level: c_int,
    /// Directory of log files.
    /// Value is passed to Glog library using FLAGS_log_dir variable.
    /// NULL means temporary directory, and "" means only writing to stderr.
    log_dir: *const c_char,
    /// prebuilt data directory. defaults to ${shared_data_dir}/build
    prebuilt_data_dir: *const c_char,
    /// staging directory. defaults to ${user_data_dir}/build
    staging_dir: *const c_char,
}

#[repr(C)]
pub struct RimeComposition {
    length: c_int,
    cursor_pos: c_int,
    sel_start: c_int,
    sel_end: c_int,
    preedit: *mut c_char,
}

#[repr(C)]
pub struct RimeCandidate {
    text: *mut c_char,
    comment: *mut c_char,
    reserved: *mut c_void,
}

#[repr(C)]
pub struct RimeMenu {
    page_size: c_int,
    page_no: c_int,
    is_last_page: Bool,
    highlighted_candidate_index: c_int,
    num_candidates: c_int,
    candidates: *mut RimeCandidate,
    select_keys: *mut c_char,
}

#[repr(C)]
pub struct RimeCommit {
    data_size: c_int,
    text: *mut c_char,
}

#[repr(C)]
pub struct RimeContext {
    data_size: c_int,
    composition: RimeComposition,
    menu: RimeMenu,
    commit_text_preview: *mut c_char,
    select_labels: *mut *mut c_char,
}

#[repr(C)]
pub struct RimeStatus {
    data_size: c_int,
    schema_id: *mut c_char,
    schema_name: *mut c_char,
    is_disabled: Bool,
    is_composing: Bool,
    is_ascii_mode: Bool,
    is_full_shape: Bool,
    is_simplified: Bool,
    is_traditional: Bool,
    is_ascii_punct: Bool,
}

#[repr(C)]
pub struct RimeCandidateListIterator {
    ptr: *mut c_void,
    index: c_int,
    candidate: RimeCandidate,
}

#[repr(C)]
pub struct RimeConfig {
    ptr: *mut c_void,
}

#[repr(C)]
pub struct RimeConfigIterator {
    list: *mut c_void,
    map: *mut c_void,
    index: c_int,
    key: *const c_char,
    path: *const c_char,
}

#[repr(C)]
pub struct RimeSchemaListItem {
    schema_id: *mut c_char,
    name: *mut c_char,
    reserved: *mut c_void,
}

#[repr(C)]
pub struct RimeSchemaList {
    size: usize,
    list: *mut RimeSchemaListItem,
}

pub type RimeNotificationHandler = extern "C" fn(
    context_object: *mut c_void,
    session_id: RimeSessionId,
    message_type: *const c_char,
    message_value: *const c_char,
);

#[repr(C)]
pub struct RimeStringSlice {
    str: *const c_char,
    length: usize,
}

#[repr(C)]
pub struct RimeCustomApi {
    data_size: c_int,
}

#[repr(C)]
pub struct RimeModule {
    data_size: c_int,
    module_name: *const c_char,
    initialize: Option<unsafe extern "C" fn()>,
    finalize: Option<unsafe extern "C" fn()>,
    get_api: Option<unsafe extern "C" fn() -> *mut RimeCustomApi>,
}

#[repr(C)]
pub struct RimeApi {
    data_size: c_int,
    setup: Option<unsafe extern "C" fn(traits: *mut RimeTraits)>,
    set_notification_handler:
        Option<extern "C" fn(handler: RimeNotificationHandler, context_object: *mut c_void)>,
    initialize: Option<unsafe extern "C" fn(traits: *mut RimeTraits)>,
    finalize: Option<unsafe extern "C" fn()>,
    start_maintenance: Option<unsafe extern "C" fn(full_check: Bool) -> Bool>,
    is_maintenance_mode: Option<unsafe extern "C" fn() -> Bool>,
    join_maintenance_thread: Option<unsafe extern "C" fn()>,
    deployer_initialize: Option<unsafe extern "C" fn(traits: *mut RimeTraits)>,
    prebuild: Option<unsafe extern "C" fn() -> Bool>,
    deploy: Option<unsafe extern "C" fn() -> Bool>,
    deploy_schema: Option<unsafe extern "C" fn(schema_file: *const c_char) -> Bool>,
    deploy_config_file:
        Option<extern "C" fn(file_name: *const c_char, version_key: *const c_char) -> Bool>,
    sync_user_data: Option<unsafe extern "C" fn() -> Bool>,
    create_session: Option<unsafe extern "C" fn() -> RimeSessionId>,
    find_session: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> Bool>,
    destroy_session: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> Bool>,
    cleanup_stale_sessions: Option<unsafe extern "C" fn()>,
    cleanup_all_sessions: Option<unsafe extern "C" fn()>,
    process_key: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, keycode: c_int, mask: c_int) -> Bool,
    >,
    commit_composition: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> Bool>,
    clear_composition: Option<unsafe extern "C" fn(session_id: RimeSessionId)>,
    get_commit:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, commit: *mut RimeCommit) -> Bool>,
    free_commit: Option<unsafe extern "C" fn(commit: *mut RimeCommit) -> Bool>,
    get_context:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, context: *mut RimeContext) -> Bool>,
    free_context: Option<unsafe extern "C" fn(ctx: *mut RimeContext) -> Bool>,
    get_status:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, status: *mut RimeStatus) -> Bool>,
    free_status: Option<unsafe extern "C" fn(status: *mut RimeStatus) -> Bool>,
    set_option:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, option: *const c_char, value: Bool)>,
    get_option:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, option: *const c_char) -> Bool>,
    set_property: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, prop: *const c_char, value: *const c_char),
    >,
    get_property: Option<
        extern "C" fn(
            session_id: RimeSessionId,
            prop: *const c_char,
            value: *mut c_char,
            buffer_size: usize,
        ) -> Bool,
    >,
    get_schema_list: Option<unsafe extern "C" fn(schema_list: *mut RimeSchemaList) -> Bool>,
    free_schema_list: Option<unsafe extern "C" fn(schema_list: *mut RimeSchemaList)>,
    get_current_schema: Option<
        extern "C" fn(
            session_id: RimeSessionId,
            schema_id: *mut c_char,
            buffer_size: usize,
        ) -> Bool,
    >,
    select_schema:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, schema_id: *const c_char) -> Bool>,
    schema_open:
        Option<unsafe extern "C" fn(schema_id: *const c_char, config: *mut RimeConfig) -> Bool>,
    config_open:
        Option<unsafe extern "C" fn(config_id: *const c_char, config: *mut RimeConfig) -> Bool>,
    config_close: Option<unsafe extern "C" fn(config: *mut RimeConfig) -> Bool>,
    config_get_bool: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: *mut Bool) -> Bool,
    >,
    config_get_int: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut c_int,
        ) -> Bool,
    >,
    config_get_double: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: *mut f64) -> Bool,
    >,
    config_get_string: Option<
        extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut c_char,
            buffer_size: usize,
        ) -> Bool,
    >,
    config_get_cstring:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> *const c_char>,
    config_update_signature:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, signer: *const c_char) -> Bool>,
    config_begin_map: Option<
        unsafe extern "C" fn(
            iterator: *mut RimeConfigIterator,
            config: *mut RimeConfig,
            key: *const c_char,
        ) -> Bool,
    >,
    config_next: Option<unsafe extern "C" fn(iterator: *mut RimeConfigIterator) -> Bool>,
    config_end: Option<unsafe extern "C" fn(iterator: *mut RimeConfigIterator)>,

    // testing
    simulate_key_sequence: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, key_sequence: *const c_char) -> Bool,
    >,

    // module
    register_module: Option<unsafe extern "C" fn(module: *mut RimeModule) -> Bool>,
    find_module: Option<unsafe extern "C" fn(module_name: *const c_char) -> *mut RimeModule>,

    run_task: Option<unsafe extern "C" fn(task_name: *const c_char) -> Bool>,

    get_user_id: Option<unsafe extern "C" fn() -> *const c_char>,
    get_user_data_sync_dir: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,

    // config initialization
    config_init: Option<unsafe extern "C" fn(config: *mut RimeConfig) -> Bool>,
    config_load_string:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, yaml: *const c_char) -> Bool>,

    // configuration setters
    config_set_bool: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: Bool) -> Bool,
    >,
    config_set_int: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: i32) -> Bool,
    >,
    config_set_double: Option<
        unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char, value: f64) -> Bool,
    >,
    config_set_string: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *const c_char,
        ) -> Bool,
    >,

    // configuration manipulation
    config_get_item: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut RimeConfig,
        ) -> Bool,
    >,
    config_set_item: Option<
        unsafe extern "C" fn(
            config: *mut RimeConfig,
            key: *const c_char,
            value: *mut RimeConfig,
        ) -> Bool,
    >,
    config_clear: Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> Bool>,
    config_create_list:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> Bool>,
    config_create_map:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> Bool>,
    config_list_size:
        Option<unsafe extern "C" fn(config: *mut RimeConfig, key: *const c_char) -> usize>,
    config_begin_list: Option<
        unsafe extern "C" fn(
            iterator: *mut RimeConfigIterator,
            config: *mut RimeConfig,
            key: *const c_char,
        ) -> Bool,
    >,

    // raw input access
    get_input: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> *const c_char>,

    // caret position
    get_caret_pos: Option<unsafe extern "C" fn(session_id: RimeSessionId) -> usize>,
    set_caret_pos: Option<unsafe extern "C" fn(session_id: RimeSessionId, caret_pos: usize)>,

    // candidate selection
    select_candidate: Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,
    select_candidate_on_current_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,

    // candidate list access
    candidate_list_begin: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            iterator: *mut RimeCandidateListIterator,
        ) -> Bool,
    >,
    candidate_list_next:
        Option<unsafe extern "C" fn(iterator: *mut RimeCandidateListIterator) -> Bool>,
    candidate_list_end: Option<unsafe extern "C" fn(iterator: *mut RimeCandidateListIterator)>,

    user_config_open:
        Option<unsafe extern "C" fn(config_id: *const c_char, config: *mut RimeConfig) -> Bool>,
    candidate_list_from_index: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            iterator: *mut RimeCandidateListIterator,
            index: i32,
        ) -> Bool,
    >,

    // protobuf
    commit_proto: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, commit_builder: *mut RimeProtoBuilder),
    >,
    context_proto: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, context_builder: *mut RimeProtoBuilder),
    >,
    status_proto: Option<
        unsafe extern "C" fn(session_id: RimeSessionId, status_builder: *mut RimeProtoBuilder),
    >,

    get_state_label: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            option_name: *const c_char,
            state: Bool,
        ) -> *const c_char,
    >,
    delete_candidate: Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,
    delete_candidate_on_current_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,

    get_state_label_abbreviated: Option<
        unsafe extern "C" fn(
            session_id: RimeSessionId,
            option_name: *const c_char,
            state: Bool,
            abbreviated: Bool,
        ) -> RimeStringSlice,
    >,
    set_input:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, input: *const c_char) -> Bool>,

    get_shared_data_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    get_user_data_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    get_prebuilt_data_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    get_staging_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,
    get_sync_dir_s: Option<unsafe extern "C" fn(dir: *mut c_char, buffer_size: usize)>,

    highlight_candidate:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,
    highlight_candidate_on_current_page:
        Option<unsafe extern "C" fn(session_id: RimeSessionId, index: usize) -> Bool>,

    change_page: Option<unsafe extern "C" fn(session_id: RimeSessionId, backward: Bool) -> Bool>,

    // version info
    get_version: Option<unsafe extern "C" fn() -> *const c_char>,
}
