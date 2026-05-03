const C = {
  reset:     "\x1b[0m",
  keyword:   "\x1b[34m",   // 青
  string:    "\x1b[32m",   // 緑
  number:    "\x1b[36m",   // シアン
  comment:   "\x1b[90m",   // 暗いグレー
  function:  "\x1b[33m",   // 黄
  type:      "\x1b[35m",   // マゼンタ
  operator:  "\x1b[37m",   // 白
  constant:  "\x1b[96m",   // 明るいシアン
  attribute: "\x1b[93m",   // 明るいイエロー
} as const;

export const THEME: Record<string, string> = {
  // --- 共通 ---
  "comment":                      C.comment,
  "line_comment":                 C.comment,
  "block_comment":                C.comment,
  "string":                       C.string,
  "string_literal":               C.string,
  "template_string":              C.string,
  "raw_string_literal":           C.string,
  "interpreted_string_literal":   C.string,
  "number":                       C.number,
  "integer":                      C.number,
  "float":                        C.number,
  "integer_literal":              C.number,
  "float_literal":                C.number,
  "true":                         C.constant,
  "false":                        C.constant,
  "null":                         C.constant,
  "none":                         C.constant,
  // --- キーワード ---
  "if":                           C.keyword,
  "else":                         C.keyword,
  "return":                       C.keyword,
  "for":                          C.keyword,
  "while":                        C.keyword,
  "break":                        C.keyword,
  "continue":                     C.keyword,
  "import":                       C.keyword,
  "export":                       C.keyword,
  "from":                         C.keyword,
  "const":                        C.keyword,
  "let":                          C.keyword,
  "var":                          C.keyword,
  "function":                     C.keyword,
  "class":                        C.keyword,
  "new":                          C.keyword,
  "type":                         C.keyword,
  "interface":                    C.keyword,
  "enum":                         C.keyword,
  "async":                        C.keyword,
  "await":                        C.keyword,
  "pub":                          C.keyword,
  "fn":                           C.keyword,
  "use_declaration":              C.keyword,
  "def":                          C.keyword,
  "lambda":                       C.keyword,
  "func":                         C.keyword,
  "package":                      C.keyword,
  // --- 型 ---
  "type_identifier":              C.type,
  "predefined_type":              C.type,
  "primitive_type":               C.type,
  "type_name":                    C.type,
  // --- 関数 ---
  "function_declaration":         C.function,
  "method_definition":            C.function,
  "function_definition":          C.function,
  // --- HTML/CSS 固有 ---
  "tag_name":                     C.keyword,
  "attribute_name":               C.attribute,
  "attribute_value":              C.string,
  "property_name":                C.attribute,
  "class_selector":               C.function,
  "id_selector":                  C.constant,
};

export const RESET = C.reset;
