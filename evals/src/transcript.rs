use serde_json::Value;

pub fn events(jsonl: &str) -> Vec<Value> {
    jsonl
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(Value::is_object)
        .collect()
}

fn parts(event: &Value) -> &[Value] {
    event["message"]["content"].as_array().map_or(&[], Vec::as_slice)
}

fn kind(event: &Value) -> &str {
    event["type"].as_str().unwrap_or_default()
}

fn clip(text: &str) -> String {
    text.chars().take(500).collect()
}

pub fn digest(jsonl: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for event in events(jsonl) {
        for part in parts(&event) {
            match (kind(&event), part["type"].as_str()) {
                ("assistant", Some("text")) => {
                    lines.push(format!("ASSISTANT: {}", part["text"].as_str().unwrap_or_default()));
                }
                ("assistant", Some("tool_use")) => lines.push(format!(
                    "TOOL {}: {}",
                    part["name"].as_str().unwrap_or_default(),
                    clip(&part["input"].to_string())
                )),
                ("user", Some("tool_result")) => {
                    let result = &part["content"];
                    let text = result.as_str().map_or_else(|| result.to_string(), String::from);
                    lines.push(format!("RESULT: {}", clip(&text)));
                }
                _ => {}
            }
        }
    }
    lines
}

pub fn said(jsonl: &str) -> Vec<String> {
    events(jsonl)
        .iter()
        .filter(|e| kind(e) == "assistant" && e["parent_tool_use_id"].is_null())
        .flat_map(parts)
        .filter(|p| p["type"] == "text")
        .filter_map(|p| p["text"].as_str().map(String::from))
        .collect()
}

pub fn session_id(jsonl: &str) -> Option<String> {
    events(jsonl)
        .iter()
        .rev()
        .find(|e| kind(e) == "result")
        .and_then(|e| e["session_id"].as_str().map(String::from))
}

pub fn invoked(jsonl: &str, skill: &str) -> bool {
    events(jsonl)
        .iter()
        .filter(|e| kind(e) == "assistant")
        .flat_map(parts)
        .any(|p| p["name"] == "Skill" && p["input"]["skill"] == skill)
}

#[cfg(test)]
mod tests;
