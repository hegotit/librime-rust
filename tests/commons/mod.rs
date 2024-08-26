pub fn enable_log() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();
}

pub fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}
