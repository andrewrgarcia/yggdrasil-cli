use crate::cli::Args;

/// What the shortcut flags (`--treed` / `--whited` / `--printed`) resolve to.
///
// viceroy: extracted from run_snapshot() — flag precedence split from
// rendering. It was four sequential mutations of args.contents; now it is one
// pure function with the precedence rules stated once and tested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPlan {
    pub contents: bool,
    pub out: Option<String>,
    /// Force interactive `--white` selection when none was given.
    pub force_white: bool,
}

const DEFAULT_OUT: &str = "SHOW.md";

fn target(opt: &Option<String>) -> String {
    opt.clone().unwrap_or_else(|| DEFAULT_OUT.to_string())
}

/// Resolve the shortcut flags into a single output plan.
///
/// Precedence, preserved exactly from the original inline logic:
///   1. `--treed`   -> index-only, markdown, interactive white
///   2. `--whited`  -> full codex, markdown, interactive white
///   3. `--printed` -> full codex, markdown
///   4. `--treed` wins the `contents` question outright if present
pub fn plan_output(
    treed: &Option<Option<String>>,
    whited: &Option<Option<String>>,
    printed: &Option<Option<String>>,
    contents: bool,
    out: Option<String>,
) -> OutputPlan {
    let mut plan = OutputPlan {
        contents,
        out,
        force_white: false,
    };

    if let Some(opt) = treed {
        plan.contents = false;
        plan.out = Some(target(opt));
        plan.force_white = true;
    }

    if let Some(opt) = whited {
        plan.contents = true;
        plan.out = Some(target(opt));
        plan.force_white = true;
    }

    if let Some(opt) = printed {
        plan.contents = true;
        plan.out = Some(target(opt));
    }

    // Index-only is non-negotiable once --treed is in play.
    if treed.is_some() {
        plan.contents = false;
    }

    plan
}

/// Apply the resolved plan back onto `args`.
pub fn apply_output_plan(args: &mut Args) {
    let plan = plan_output(
        &args.treed,
        &args.whited,
        &args.printed,
        args.contents,
        args.out.clone(),
    );

    args.contents = plan.contents;
    args.out = plan.out;

    if plan.force_white && args.white.is_none() {
        args.white = Some(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(
        treed: Option<Option<&str>>,
        whited: Option<Option<&str>>,
        printed: Option<Option<&str>>,
    ) -> OutputPlan {
        let conv = |o: Option<Option<&str>>| {
            o.map(|inner| inner.map(|s| s.to_string()))
        };
        plan_output(&conv(treed), &conv(whited), &conv(printed), false, None)
    }

    #[test]
    fn no_shortcuts_is_a_no_op() {
        let p = plan(None, None, None);
        assert_eq!(p.contents, false);
        assert_eq!(p.out, None);
        assert_eq!(p.force_white, false);
    }

    #[test]
    fn printed_writes_full_codex_to_show_md() {
        let p = plan(None, None, Some(None));
        assert!(p.contents);
        assert_eq!(p.out.as_deref(), Some("SHOW.md"));
        assert!(!p.force_white);
    }

    #[test]
    fn whited_is_interactive_full_codex() {
        let p = plan(None, Some(None), None);
        assert!(p.contents);
        assert!(p.force_white);
    }

    #[test]
    fn treed_is_index_only_and_interactive() {
        let p = plan(Some(None), None, None);
        assert!(!p.contents);
        assert!(p.force_white);
        assert_eq!(p.out.as_deref(), Some("SHOW.md"));
    }

    #[test]
    fn treed_beats_printed_on_contents() {
        // This is the case the old sequential mutation nearly lost.
        let p = plan(Some(None), None, Some(None));
        assert!(!p.contents, "--treed must stay index-only");
    }

    #[test]
    fn explicit_names_win() {
        let p = plan(None, None, Some(Some("MyFile.md")));
        assert_eq!(p.out.as_deref(), Some("MyFile.md"));
    }
}