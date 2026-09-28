use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LanguageInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub lsp_server: Option<&'static str>,
    pub icon: &'static str,
}

pub static LANGUAGES: &[LanguageInfo] = &[
    LanguageInfo {
        id: "javascript",
        name: "JavaScript",
        extensions: &["js", "mjs", "cjs"],
        lsp_server: Some("typescript-language-server"),
        icon: "file_icons/file_type_js.svg",
    },
    LanguageInfo {
        id: "typescript",
        name: "TypeScript",
        extensions: &["ts", "mts", "cts"],
        lsp_server: Some("typescript-language-server"),
        icon: "file_icons/file_type_typescript.svg",
    },
    LanguageInfo {
        id: "tsx",
        name: "TypeScript React (TSX)",
        extensions: &["tsx"],
        lsp_server: Some("typescript-language-server"),
        icon: "file_icons/file_type_reactts.svg",
    },
    LanguageInfo {
        id: "jsx",
        name: "JavaScript React (JSX)",
        extensions: &["jsx"],
        lsp_server: Some("typescript-language-server"),
        icon: "file_icons/file_type_reactjs.svg",
    },
    LanguageInfo {
        id: "python",
        name: "Python",
        extensions: &["py", "pyw"],
        lsp_server: Some("basedpyright-langserver"),
        icon: "file_icons/file_type_python.svg",
    },
    LanguageInfo {
        id: "rust",
        name: "Rust",
        extensions: &["rs"],
        lsp_server: Some("rust-analyzer"),
        icon: "file_icons/file_type_rust.svg",
    },
    LanguageInfo {
        id: "go",
        name: "Go",
        extensions: &["go"],
        lsp_server: Some("gopls"),
        icon: "file_icons/file_type_go.svg",
    },
    LanguageInfo {
        id: "c",
        name: "C",
        extensions: &["c", "h"],
        lsp_server: Some("clangd"),
        icon: "file_icons/file_type_c.svg",
    },
    LanguageInfo {
        id: "cpp",
        name: "C++",
        extensions: &["cpp", "cc", "cxx", "hpp", "hh", "hxx", "inl"],
        lsp_server: Some("clangd"),
        icon: "file_icons/file_type_cpp.svg",
    },
    LanguageInfo {
        id: "csharp",
        name: "C#",
        extensions: &["cs"],
        lsp_server: Some("csharp-ls"),
        icon: "file_icons/file_type_csharp.svg",
    },
    LanguageInfo {
        id: "html",
        name: "HTML",
        extensions: &["html", "htm", "xhtml", "vue", "svelte", "astro"],
        lsp_server: Some("vscode-html-language-server"),
        icon: "file_icons/file_type_html.svg",
    },
    LanguageInfo {
        id: "css",
        name: "CSS",
        extensions: &["css", "scss", "sass", "less"],
        lsp_server: Some("vscode-css-language-server"),
        icon: "file_icons/file_type_css.svg",
    },
    LanguageInfo {
        id: "json",
        name: "JSON",
        extensions: &["json", "jsonc", "json5"],
        lsp_server: Some("json-language-server"),
        icon: "file_icons/file_type_json.svg",
    },
    LanguageInfo {
        id: "yaml",
        name: "YAML",
        extensions: &["yaml", "yml"],
        lsp_server: Some("yaml-language-server"),
        icon: "file_icons/file_type_yaml.svg",
    },
    LanguageInfo {
        id: "toml",
        name: "TOML",
        extensions: &["toml"],
        lsp_server: Some("taplo"),
        icon: "file_icons/file_type_toml.svg",
    },
    LanguageInfo {
        id: "bash",
        name: "Shell Script (Bash)",
        extensions: &["sh", "bash", "zsh", "ps1", "psm1"],
        lsp_server: Some("bash-language-server"),
        icon: "file_icons/file_type_shell.svg",
    },
    LanguageInfo {
        id: "markdown",
        name: "Markdown",
        extensions: &["md", "markdown", "mdown"],
        lsp_server: Some("marksman"),
        icon: "file_icons/file_type_markdown.svg",
    },
    LanguageInfo {
        id: "dockerfile",
        name: "Dockerfile",
        extensions: &["dockerfile"],
        lsp_server: Some("docker-langserver"),
        icon: "file_icons/file_type_docker.svg",
    },
    LanguageInfo {
        id: "zig",
        name: "Zig",
        extensions: &["zig"],
        lsp_server: Some("zls"),
        icon: "file_icons/file_type_zig.svg",
    },
    LanguageInfo {
        id: "lua",
        name: "Lua",
        extensions: &["lua", "luau"],
        lsp_server: Some("lua-language-server"),
        icon: "file_icons/file_type_lua.svg",
    },
    LanguageInfo {
        id: "php",
        name: "PHP",
        extensions: &["php"],
        lsp_server: Some("intelephense"),
        icon: "file_icons/file_type_php.svg",
    },
    LanguageInfo {
        id: "ruby",
        name: "Ruby",
        extensions: &["rb", "erb"],
        lsp_server: Some("ruby-lsp"),
        icon: "file_icons/file_type_ruby.svg",
    },
    LanguageInfo {
        id: "java",
        name: "Java",
        extensions: &["java", "jar", "class", "jsp"],
        lsp_server: Some("jdtls"),
        icon: "file_icons/file_type_java.svg",
    },
    LanguageInfo {
        id: "kotlin",
        name: "Kotlin",
        extensions: &["kt", "kts"],
        lsp_server: Some("kotlin-language-server"),
        icon: "file_icons/file_type_kotlin.svg",
    },
    LanguageInfo {
        id: "swift",
        name: "Swift",
        extensions: &["swift"],
        lsp_server: Some("sourcekit-lsp"),
        icon: "file_icons/file_type_swift.svg",
    },
    LanguageInfo {
        id: "scala",
        name: "Scala",
        extensions: &["scala", "sc"],
        lsp_server: None,
        icon: "file_icons/file_type_scala.svg",
    },
    LanguageInfo {
        id: "sql",
        name: "SQL",
        extensions: &["sql"],
        lsp_server: Some("sqlls"),
        icon: "file_icons/file_type_sql.svg",
    },
    LanguageInfo {
        id: "graphql",
        name: "GraphQL",
        extensions: &["graphql", "gql"],
        lsp_server: Some("graphql-lsp"),
        icon: "file_icons/file_type_graphql.svg",
    },
    LanguageInfo {
        id: "elixir",
        name: "Elixir",
        extensions: &["ex", "exs"],
        lsp_server: Some("elixir-ls"),
        icon: "file_icons/file_type_elixir.svg",
    },
    LanguageInfo {
        id: "diff",
        name: "Diff",
        extensions: &["diff", "patch"],
        lsp_server: None,
        icon: "file_icons/file_type_git.svg",
    },
    LanguageInfo {
        id: "cmake",
        name: "CMake",
        extensions: &["cmake"],
        lsp_server: None,
        icon: "file_icons/file_type_config.svg",
    },
    LanguageInfo {
        id: "make",
        name: "Makefile",
        extensions: &["mk", "mak"],
        lsp_server: None,
        icon: "file_icons/file_type_config.svg",
    },
    LanguageInfo {
        id: "text",
        name: "Plain Text",
        extensions: &["txt", "text", "log"],
        lsp_server: None,
        icon: "file_icons/file_type_text.svg",
    },
];

pub fn all_languages() -> &'static [LanguageInfo] {
    LANGUAGES
}

pub fn language_info(id: &str) -> Option<&'static LanguageInfo> {
    LANGUAGES.iter().find(|l| l.id.eq_ignore_ascii_case(id))
}

pub fn language_name(id: &str) -> &'static str {
    language_info(id).map(|l| l.name).unwrap_or(id)
}

pub fn language_icon(id: &str) -> &'static str {
    language_info(id)
        .map(|l| l.icon)
        .unwrap_or("file_icons/file_type_text.svg")
}

pub fn language_for(path: &Path) -> Option<&'static str> {
    let ext_raw = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    // Extensions are almost always lowercase; only allocate when an
    // uppercase letter actually needs folding (this runs on every render
    // for the status bar's language label).
    let folded: String;
    let ext = if ext_raw.bytes().any(|b| b.is_ascii_uppercase()) {
        folded = ext_raw.to_ascii_lowercase();
        folded.as_str()
    } else {
        ext_raw
    };
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match name {
        "Dockerfile" | "Containerfile" => return Some("dockerfile"),
        "Makefile" | "makefile" | "GNUmakefile" => return Some("make"),
        "CMakeLists.txt" => return Some("cmake"),
        "Justfile" | "justfile" => return Some("make"),
        ".bashrc" | ".bash_profile" | ".zshrc" | ".profile" => return Some("bash"),
        _ => {}
    }
    if let Some(lang) = by_extension(ext) {
        return Some(lang);
    }

    shebang_language(path)
}

fn by_extension(ext: &str) -> Option<&'static str> {
    for lang in LANGUAGES {
        if lang.extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)) {
            return Some(lang.id);
        }
    }

    // Secondary fallback for rarer extensions
    Some(match ext {
        "r" => "r",
        "xml" | "xsl" | "xsd" | "svg" => "xml",
        "proto" => "proto",
        "ejs" => "ejs",
        "tex" => "latex",
        "dart" => "dart",
        "hs" => "haskell",
        "ml" => "ocaml",
        "fs" | "fsx" => "fsharp",
        "erl" => "erlang",
        "clj" | "cljs" => "clojure",
        _ => return None,
    })
}

/// Detect the language of an extension-less file from its `#!` shebang.
///
/// Only the first bytes of the file can contain a shebang, so this reads a
/// single 256-byte block instead of the whole file — detection stays O(1)
/// even for huge extension-less files (minified bundles, logs, dumps).
fn shebang_language(path: &Path) -> Option<&'static str> {
    use std::io::Read as _;
    let mut file = std::fs::File::open(path).ok()?;
    let mut head = [0u8; 256];
    let n = file.read(&mut head).ok()?;
    let head = head.get(..n)?;
    if head.len() < 2 || head[0] != b'#' || head[1] != b'!' {
        return None;
    }
    let line = String::from_utf8_lossy(head).lines().next()?.to_string();
    let lower = line.to_ascii_lowercase();
    if lower.contains("python") {
        Some("python")
    } else if lower.contains("ruby") {
        Some("ruby")
    } else if lower.contains("node") || lower.contains("deno") || lower.contains("bun") {
        Some("javascript")
    } else if lower.contains("bash") || lower.contains("sh") || lower.contains("zsh") {
        Some("bash")
    } else if lower.contains("perl") {
        Some("perl")
    } else if lower.contains("php") {
        Some("php")
    } else if lower.contains("lua") {
        Some("lua")
    } else if lower.contains("elixir") {
        Some("elixir")
    } else if lower.contains("swift") {
        Some("swift")
    } else if lower.contains("go") && line.contains("go run") {
        Some("go")
    } else {
        None
    }
}

pub fn lsp_server_for(lang: &str) -> Option<&'static str> {
    if let Some(adapter) = crate::lsp::adapter::adapter_for_language(lang) {
        return Some(adapter.name);
    }
    language_info(lang).and_then(|l| l.lsp_server)
}

const TSX_HIGHLIGHT_QUERY: &str = r#"
; Types & Variables
(identifier) @variable

; Properties
(property_identifier) @property
(shorthand_property_identifier) @property
(shorthand_property_identifier_pattern) @property
(private_property_identifier) @property

; Function and method definitions
(function_expression name: (identifier) @function)
(function_declaration name: (identifier) @function)
(method_definition name: (property_identifier) @function)
(pair
  key: (property_identifier) @function
  value: [(function_expression) (arrow_function)])
(assignment_expression
  left: (member_expression property: (property_identifier) @function)
  right: [(function_expression) (arrow_function)])
(variable_declarator
  name: (identifier) @function
  value: [(function_expression) (arrow_function)])
(assignment_expression
  left: (identifier) @function
  right: [(function_expression) (arrow_function)])

; Function and method calls
(call_expression function: (identifier) @function)
(call_expression
  function: (member_expression property: (property_identifier) @function))

; Special identifiers
((identifier) @type (#match? @type "^[A-Z]"))
([
  (identifier)
  (shorthand_property_identifier)
  (shorthand_property_identifier_pattern)
 ] @constant (#match? @constant "^_*[A-Z_][A-Z\\d_]*$"))

((identifier) @variable
 (#match? @variable "^(arguments|module|console|window|document|process)$"))

; Literals
(this) @variable
(super) @variable
[(true) (false) (null) (undefined)] @constant

(comment) @comment

[(string) (template_string)] @string
(regex) @string.special
(number) @number

; JSX Elements & Attributes
(jsx_opening_element name: (_) @tag)
(jsx_closing_element name: (_) @tag)
(jsx_self_closing_element name: (_) @tag)
(jsx_attribute (property_identifier) @attribute)

; Punctuation & Delimiters
[";" (optional_chain) "." ","] @punctuation.delimiter
["-" "--" "-=" "+" "++" "+=" "*" "*=" "**" "**=" "/" "/=" "%" "%="
 "<" "<=" "<<" "<<=" "=" "==" "===" "!" "!=" "!==" "=>"
 ">" ">=" ">>" ">>=" ">>>" ">>>=" "~" "^" "&" "|" "^=" "&=" "|="
 "&&" "||" "??" "&&=" "||=" "??="] @operator

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
(template_substitution "${" @punctuation.special "}" @punctuation.special) @embedded

; Keywords
[
  "as" "async" "await" "break" "case" "catch" "class" "const"
  "continue" "debugger" "default" "delete" "do" "else" "export"
  "extends" "finally" "for" "from" "function" "get" "if" "import"
  "in" "instanceof" "let" "new" "of" "return" "set" "static"
  "switch" "target" "throw" "try" "typeof" "var" "void" "while"
  "with" "yield" "abstract" "declare" "enum" "implements"
  "interface" "keyof" "namespace" "private" "protected" "public"
  "type" "readonly" "override" "satisfies"
] @keyword

; Types
(type_identifier) @type
(predefined_type) @type
(type_arguments "<" @punctuation.bracket ">" @punctuation.bracket)
(required_parameter (identifier) @variable)
(optional_parameter (identifier) @variable)
"#;

/// Initializes and registers high-quality Tree-Sitter language definitions for TSX/TypeScript/JSX and aliases.
pub fn init_languages() {
    use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};
    let registry = LanguageRegistry::singleton();

    let tsx_config = LanguageConfig::new(
        "tsx",
        tree_sitter_typescript::LANGUAGE_TSX.into(),
        vec!["html".into(), "css".into(), "javascript".into(), "typescript".into()],
        TSX_HIGHLIGHT_QUERY,
        "",
        tree_sitter_typescript::LOCALS_QUERY,
    );
    registry.register("tsx", &tsx_config);

    // `.jsx` is its own language id (so the LSP layer can send the
    // "javascriptreact" language id), but syntactically it is TSX minus the
    // type annotations — the TSX grammar highlights it correctly.
    registry.register("jsx", &tsx_config);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_extensions() {
        assert_eq!(language_for(Path::new("main.rs")), Some("rust"));
        assert_eq!(language_for(Path::new("app.tsx")), Some("tsx"));
        assert_eq!(language_for(Path::new("x.cs")), Some("csharp"));
        assert_eq!(language_for(Path::new("Dockerfile")), Some("dockerfile"));
        assert_eq!(language_for(Path::new("CMakeLists.txt")), Some("cmake"));
        assert_eq!(language_for(Path::new("Makefile")), Some("make"));
        assert_eq!(language_for(Path::new("unknown.xyz")), None);
    }

    #[test]
    fn test_init_languages() {
        init_languages();
        let registry = gpui_component::highlighter::LanguageRegistry::singleton();
        assert!(registry.language("tsx").is_some());
        assert!(registry.language("jsx").is_some());
    }

    #[test]
    fn jsx_is_its_own_language_id() {
        // So the LSP layer can map it to "javascriptreact".
        assert_eq!(language_for(Path::new("App.jsx")), Some("jsx"));
        assert_eq!(language_for(Path::new("app.js")), Some("javascript"));
    }

    #[test]
    fn web_languages_resolve_to_a_server() {
        assert_eq!(
            lsp_server_for("tsx"),
            Some("typescript-language-server")
        );
        assert_eq!(lsp_server_for("css"), Some("vscode-css-language-server"));
        assert_eq!(lsp_server_for("html"), Some("vscode-html-language-server"));
        assert_eq!(lsp_server_for("json"), Some("json-language-server"));
        assert_eq!(lsp_server_for("rust"), Some("rust-analyzer"));
        assert_eq!(lsp_server_for("python"), Some("basedpyright-langserver"));
        assert_eq!(lsp_server_for("go"), Some("gopls"));
        assert_eq!(lsp_server_for("plaintext"), None);
    }

    #[test]
    fn test_language_names_and_metadata() {
        assert_eq!(language_name("rust"), "Rust");
        assert_eq!(language_name("javascript"), "JavaScript");
        assert_eq!(language_name("python"), "Python");
        assert_eq!(language_name("go"), "Go");
        assert_eq!(language_name("typescript"), "TypeScript");

        assert!(all_languages().len() >= 25);
    }
}
