use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use serde::Serialize;
use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
#[derive(Serialize, Debug)]
pub struct MatchResult {
    pattern_index: usize,
    start: usize,
    end: usize,
}


#[wasm_bindgen]
pub struct AhoSearcher {
    ac: AhoCorasick,
}

#[wasm_bindgen]
impl AhoSearcher {
    
    #[wasm_bindgen(constructor)]
    pub fn new(patterns: JsValue) -> Result<AhoSearcher, JsValue> {
        
        let patterns_vec: Vec<String> = serde_wasm_bindgen::from_value(patterns)
            .map_err(|e| JsValue::from_str(&format!("初始化失败：需要一个字符串数组。 {}", e)))?;

        
        let ac = AhoCorasickBuilder::new()
            .match_kind(MatchKind::Standard)
            .build(&patterns_vec)
            .map_err(|e| JsValue::from_str(&format!("构建 AhoCorasick 失败：{}", e)))?;

        
        Ok(AhoSearcher { ac })
    }
    
    pub fn search(&self, haystack: &str) -> Result<JsValue, JsValue> {
        
        let mut matches = Vec::new();
        for mat in self.ac.find_iter(haystack) {
            matches.push(MatchResult {
                pattern_index: mat.pattern().as_usize(),
                start: mat.start(),
                end: mat.end(),
            });
        }
        serde_wasm_bindgen::to_value(&matches)
            .map_err(|e| JsValue::from_str(&format!("序列化匹配结果失败：{}", e)))
    }
}