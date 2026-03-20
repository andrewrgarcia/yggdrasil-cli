/// Extract static top-level imports from a Python file.
///
/// Reads only the preamble — stops at the first non-import, non-blank,
/// non-comment, non-docstring line that is also outside a parenthesised
/// continuation block.
///
/// Handles:
///   import a, b, c
///   import a.b.c as foo
///   from x.y import z
///   from x.y import (        ← multi-line parens
///       A,
///       B,
///   )
///
/// Returns raw module specifiers, e.g. ["os", "a.b.c", "x.y"].
pub fn extract_python_imports(source: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let mut in_docstring = false;
    let mut docstring_delim = "";
    let mut in_parens = false;   // inside a multi-line import (...)

    for line in source.lines() {
        let trimmed = line.trim();

        // ── docstring handling ─────────────────────────────────────────
        if in_docstring {
            if trimmed.contains(docstring_delim) {
                in_docstring = false;
            }
            continue;
        }

        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("r\"\"\"") {
            let stripped = trimmed.trim_start_matches('r').trim_start_matches('"');
            if !stripped.contains("\"\"\"") {
                in_docstring = true;
                docstring_delim = "\"\"\"";
            }
            continue;
        }

        if trimmed.starts_with("'''") || trimmed.starts_with("r'''") {
            let stripped = trimmed.trim_start_matches('r').trim_start_matches('\'');
            if !stripped.contains("'''") {
                in_docstring = true;
                docstring_delim = "'''";
            }
            continue;
        }

        // ── inside a parenthesised continuation ───────────────────────
        // e.g. the lines between  `from foo import (` and the closing `)`
        if in_parens {
            if trimmed.contains(')') {
                in_parens = false;
            }
            // these lines are names/aliases, not module specifiers — skip
            continue;
        }

        // ── skip blanks and comments ───────────────────────────────────
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // ── import <module> ────────────────────────────────────────────
        if let Some(rest) = trimmed.strip_prefix("import ") {
            for part in rest.split(',') {
                let module = part
                    .trim()
                    .split_whitespace()
                    .next()           // drop "as alias"
                    .unwrap_or("")
                    .to_string();
                if !module.is_empty() {
                    imports.push(module);
                }
            }
            continue;
        }

        // ── from <module> import <names> ───────────────────────────────
        if let Some(rest) = trimmed.strip_prefix("from ") {
            if let Some(module) = rest.split_whitespace().next() {
                if !module.starts_with('.') {
                    imports.push(module.to_string());
                }
            }
            // open paren on the same line → continuation follows
            if trimmed.contains('(') && !trimmed.contains(')') {
                in_parens = true;
            }
            continue;
        }

        // ── anything else (outside parens) stops the preamble scan ────
        break;
    }

    imports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_imports() {
        let src = "import os\nimport sys\nfrom pathlib import Path\nfrom a.b.c import Thing\n\nx = 1\n";
        let got = extract_python_imports(src);
        assert_eq!(got, vec!["os", "sys", "pathlib", "a.b.c"]);
    }

    #[test]
    fn test_alias_import() {
        let src = "import numpy as np\nfrom foo.bar import baz as qux\n";
        let got = extract_python_imports(src);
        assert_eq!(got, vec!["numpy", "foo.bar"]);
    }

    #[test]
    fn test_multi_import() {
        let src = "import os, sys, pathlib\n";
        let got = extract_python_imports(src);
        assert_eq!(got, vec!["os", "sys", "pathlib"]);
    }

    #[test]
    fn test_stops_at_code() {
        let src = "import os\n\ndef foo():\n    import hidden\n";
        let got = extract_python_imports(src);
        assert_eq!(got, vec!["os"]);
    }

    #[test]
    fn test_skips_relative() {
        let src = "from .sibling import x\nfrom ..parent import y\nimport real\n";
        let got = extract_python_imports(src);
        assert_eq!(got, vec!["real"]);
    }

    #[test]
    fn test_comment_and_blank() {
        let src = "# header\nimport os\n\nimport sys\n";
        let got = extract_python_imports(src);
        assert_eq!(got, vec!["os", "sys"]);
    }

    #[test]
    fn test_multiline_parens() {
        let src = "\
import os
import numpy as np
import pandas as pd
from graveyard.configs.macro_config import CONFIG
from graveyard.meta.preprocess import run_all, plot_last_reporting_dates
from graveyard.meta.selector import (
    RecursiveSelector,
    SelectorConfig,
    surrogate_selector_run,
)
from graveyard.meta.selector.visualizations.core import (
    load_repeat_mask_from_yaml,
    plot_trial_and_meta_losses,
)
from graveyard.meta.metadata_builder import build_n_process_metadata
from series_xai.shap_explainers import explain_macrovars, explain_metadata
";
        let got = extract_python_imports(src);
        assert_eq!(got, vec![
            "os",
            "numpy",
            "pandas",
            "graveyard.configs.macro_config",
            "graveyard.meta.preprocess",
            "graveyard.meta.selector",
            "graveyard.meta.selector.visualizations.core",
            "graveyard.meta.metadata_builder",
            "series_xai.shap_explainers",
        ]);
    }
}