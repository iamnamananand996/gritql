use crate::binding::{linearize_binding, Binding, Constant};

use crate::pattern::resolved_pattern::CodeRange;
use crate::pattern::state::FileRegistry;
use crate::pattern::Effect;
use crate::suppress::is_binding_suppressed;
use anyhow::Result;
use im::Vector;
use marzano_language::target_language::TargetLanguage;
use marzano_util::analysis_logs::AnalysisLogs;
use std::collections::HashMap;
use std::ops::Range;

/**
 * Applies the given effects to the given code, using the bindings to resolve metavariables in the snippets.
 *
 * Bindings is a mapping from variable names to replacement string -- which is obtained from any of the nodes in the bindings vector.
 */

#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_effects<'a>(
    code: &'a str,
    effects: Vector<Effect<'a>>,
    files: &FileRegistry<'a>,
    the_filename: &str,
    new_filename: &mut Constant,
    language: &TargetLanguage,
    current_name: &Option<String>,
    logs: &mut AnalysisLogs,
) -> Result<(String, Option<Vec<Range<usize>>>)> {
    let mut our_effects = Vec::new();
    for effect in effects {
        let disabled = is_binding_suppressed(&effect.binding, language, current_name)?;
        if !disabled {
            our_effects.push(effect);
        }
    }
    if our_effects.is_empty() {
        return Ok((code.to_string(), None));
    }
    let mut memo: HashMap<CodeRange, Option<String>> = HashMap::new();
    let (from_inline, ranges) = linearize_binding(
        language,
        &our_effects,
        files,
        &mut memo,
        code,
        CodeRange::new(0, code.len() as u32, code),
        language.should_pad_snippet().then_some(0),
        logs,
    )?;
    for effect in our_effects.iter() {
        if let Binding::FileName(c) = effect.binding {
            if std::ptr::eq(c, the_filename) {
                let snippet = effect.pattern.linearized_text(
                    language,
                    &our_effects,
                    files,
                    &mut memo,
                    false,
                    logs,
                )?;
                *new_filename = Constant::String(snippet.to_string());
            }
        }
    }
    Ok((from_inline.to_string(), Some(ranges)))
}
