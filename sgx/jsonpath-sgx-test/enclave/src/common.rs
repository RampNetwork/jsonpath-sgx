extern crate env_logger;
extern crate jsonpath_lib as jsonpath;
extern crate serde_json;

use serde_json::Value;
use self::jsonpath::Selector;

use std::io::Read;
use std::string::String;
use std::vec::Vec;
use std::untrusted::fs;
use std::path::Path;

const BENCHMARK_PATH: &str = "../../../benchmark";

#[allow(dead_code)]
pub fn setup() {
    let _ = env_logger::try_init();
}

#[allow(dead_code)]
pub fn read_json(file_name: &str) -> Value {
    let path = Path::new(BENCHMARK_PATH).join(file_name);
    let mut f = fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).unwrap()
}

#[allow(dead_code)]
pub fn read_contents(file_name: &str) -> String {
    let path = Path::new(BENCHMARK_PATH).join(file_name);
    let mut f = fs::File::open(path).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    contents
}

#[allow(dead_code)]
pub fn select_and_then_compare(path: &str, json: Value, target: Value) {
    let mut selector = Selector::default();
    let result = selector
        .str_path(path)
        .unwrap()
        .value(&json)
        .select_as::<Value>()
        .unwrap();
    assert_eq!(
        result,
        match target {
            Value::Array(vec) => vec.clone(),
            _ => panic!("Give me the Array!"),
        },
        "{}",
        path
    );
}

#[allow(dead_code)]
pub fn compare_result(result: Vec<&Value>, target: Value) {
    let result = serde_json::to_value(result).unwrap();
    assert_eq!(result, target);
}
