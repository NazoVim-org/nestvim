use std::path::Path;

pub fn detect_language(file_path: Option<&Path>) -> &'static str {
    let Some(path) = file_path else {
        return "Plain Text";
    };
    
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    match ext {
        "rs" => "Rust",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "py" => "Python",
        "rb" => "Ruby",
        "go" => "Go",
        "java" => "Java",
        "c" | "h" => "C",
        "cpp" | "cc" | "hpp" => "C++",
        "sh" | "bash" => "Shell",
        "md" => "Markdown",
        "json" => "JSON",
        "toml" => "TOML",
        _ => "Plain Text",
    }
}
