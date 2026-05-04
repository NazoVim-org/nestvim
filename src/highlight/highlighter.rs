use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use std::sync::{Arc, Mutex};

pub struct Highlighter {
    syntax_set: Arc<SyntaxSet>,
    theme_set: ThemeSet,
    cache: Arc<Mutex<Option<Vec<Vec<(Style, String)>>>>>,
    dirty: bool,
}

impl Highlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        Self {
            syntax_set: Arc::new(syntax_set),
            theme_set,
            cache: Arc::new(Mutex::new(None)),
            dirty: true,
        }
    }

    pub async fn init(&self) -> Result<(), String> {
        // Initialization is done in new()
        Ok(())
    }

    pub fn set_language(&self, _lang: &str) {
        // Language setting is handled during highlighting
        self.mark_dirty();
    }

    pub fn mark_dirty(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = None;
        // Note: We can't set dirty flag on &self, would need mutable ref
    }

    pub async fn update(&self, text: &str, file_path: Option<&std::path::Path>) {
        let syntax_set = self.syntax_set.clone();
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        
        let syntax = file_path
            .and_then(|p| {
                syntax_set
                    .find_syntax_for_file(p)
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
        
        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();
        
        for line in LinesWithEndings::from(text) {
            let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &syntax_set).unwrap_or_default();
            let styled: Vec<(Style, String)> = ranges
                .into_iter()
                .map(|(style, s)| (style, s.to_string()))
                .collect();
            result.push(styled);
        }
        
        let mut cache = self.cache.lock().unwrap();
        *cache = Some(result);
    }

    pub fn get_cache(&self) -> Option<Vec<Vec<(Style, String)>>> {
        let cache = self.cache.lock().unwrap();
        cache.clone()
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}
