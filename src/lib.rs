use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use serde::Serialize;
use wasm_bindgen::prelude::*;

// (可选) 设置 panic 钩子，以便在 Wasm 发生 panic 时在 JS 控制台中获得更好的错误消息
#[wasm_bindgen(start)]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// 定义我们将从 Rust 返回给 JavaScript 的匹配结果结构。
/// `Serialize` 允许 `serde-wasm-bindgen` 自动将其转换为 JS 对象。
#[derive(Serialize, Debug)]
pub struct MatchResult {
    /// 匹配的关键词在 patterns 数组中的索引
    pattern_index: usize,
    /// 匹配的开始字节位置
    start: usize,
    /// 匹配的结束字节位置
    end: usize,
}

/// 这是我们将暴露给 JavaScript 的主 "类"
#[wasm_bindgen]
pub struct AhoSearcher {
    // AhoCorasick 自动机
    // 注意：我们不能直接暴露 AhoCorasick 类型，
    // 所以我们将它包装在自己的结构体中。
    ac: AhoCorasick,
}

#[wasm_bindgen]
impl AhoSearcher {
    /// 构造函数 (Constructor)
    ///
    /// 从 JavaScript 接收一个 `JsValue`（它应该是一个字符串数组），
    /// 并用它来构建 AhoCorasick 自动机。
    ///
    /// 在 JS 中，你可以这样调用它：
    /// `new AhoSearcher(['keyword1', 'hello', 'world'])`
    ///
    /// 我们返回 Result<..., JsValue> 以便在构建失败时
    /// （例如，传入的不是数组）向 JavaScript 抛出错误。
    #[wasm_bindgen(constructor)]
    pub fn new(patterns: JsValue) -> Result<AhoSearcher, JsValue> {
        // 1. 使用 serde-wasm-bindgen 将 JSValue (JS 数组) 反序列化为 Rust 的 Vec<String>
        let patterns_vec: Vec<String> = serde_wasm_bindgen::from_value(patterns)
            .map_err(|e| JsValue::from_str(&format!("初始化失败：需要一个字符串数组。 {}", e)))?;

        // 2. 构建 AhoCorasick 自动机
        //    我们在这里可以配置它，例如设置匹配种类
        let ac = AhoCorasickBuilder::new()
            .match_kind(MatchKind::Standard) // 标准匹配（非左长）
            .build(&patterns_vec)
            .map_err(|e| JsValue::from_str(&format!("构建 AhoCorasick 失败：{}", e)))?;

        // 3. 返回包含已构建自动机的实例
        Ok(AhoSearcher { ac })
    }

    /// 搜索方法
    ///
    /// 在给定的文本 (haystack) 中查找所有匹配项。
    /// 返回一个 `JsValue`，它将是一个 `MatchResult` 对象的数组。
    ///
    /// 在 JS 中，你可以这样调用它：
    /// `const matches = searcher.search('some text containing hello');`
    /// `console.log(matches); // [{ pattern_index: 1, start: 21, end: 26 }]`
    pub fn search(&self, haystack: &str) -> Result<JsValue, JsValue> {
        // 1. 执行搜索并收集结果
        let mut matches = Vec::new();
        for mat in self.ac.find_iter(haystack) {
            matches.push(MatchResult {
                pattern_index: mat.pattern().as_usize(),
                start: mat.start(),
                end: mat.end(),
            });
        }

        // 2. 使用 serde-wasm-bindgen 将 Rust 的 Vec<MatchResult>
        //    序列化为 JSValue (一个 JS 数组)
        serde_wasm_bindgen::to_value(&matches)
            .map_err(|e| JsValue::from_str(&format!("序列化匹配结果失败：{}", e)))
    }
}