//! BP-12, BP-16, BP-16a, BP-25. The propose and
//! diff commands. Bluepencil never writes to the
//! files it judges or reads its evidence from. The
//! parse reproduces the Python's argparse behavior
//! for this grammar byte for byte: its help texts,
//! its error shapes, and the order they fire in.
//! Help fires at the end of the token that carries
//! it; a scan error fires when its token is
//! processed and outranks a pending help; extras
//! are only reported after the required check.
//! Argument tokens are carried as text in the
//! operating system's own decoding (fsdecode), so
//! argument bytes survive as the Python's do.

use std::io::Write;

use crate::emit::{dumps_document, encode_text, Mode};
use crate::failure::Failure;
use crate::helptext::*;
use crate::inventory::parse_rules;
use crate::observations::{decode_utf8_sig, load_observations};
use crate::pyrepr::repr_text;
use crate::util::{fsdecode_units, stderr_render};
use crate::value::Text;

/// One argument token: its text, and its decoded
/// scalars for the parser's character work.
struct Token {
    text: Text,
    scalars: Vec<u32>,
}

impl Token {
    fn from_units(units: Vec<u16>) -> Token {
        let scalars = crate::util::scalars_of(&units);
        Token {
            text: Text::from_units(units),
            scalars,
        }
    }

    fn rendered(&self) -> String {
        stderr_render(self.text.units())
    }
}

/// Whether a token is shaped like an option: it
/// starts with a dash, is not a bare dash, and does
/// not look like a negative number (no option in
/// this grammar takes one, so argparse lets those
/// through as values).
fn option_shaped(token: &Token) -> bool {
    let scalars = &token.scalars;
    if scalars.len() < 2 || scalars[0] != '-' as u32 {
        return false;
    }
    !is_negative_number(&scalars[1..])
}

/// argparse's negative number matcher, the full
/// token pattern ^-\d+$|^-\d*\.\d+$ with \d a
/// Unicode decimal digit: a token matching it
/// whole is a value, never an option.
fn is_negative_number(rest: &[u32]) -> bool {
    let digit = |c: &u32| crate::util::decimal_digit(*c).is_some();
    if !rest.is_empty() && rest.iter().all(digit) {
        return true;
    }
    match rest.iter().position(|c| *c == '.' as u32) {
        Some(dot) => {
            let (before, after) = (&rest[..dot], &rest[dot + 1..]);
            !after.is_empty() && before.iter().all(digit) && after.iter().all(digit)
        }
        None => false,
    }
}

fn scalars_of_str(text: &str) -> Vec<u32> {
    text.chars().map(|ch| ch as u32).collect()
}

/// Split a long-option token at its first '=': the
/// name's scalars, and the glued value's text when
/// an '=' is present.
fn split_long(token: &Token) -> (&[u32], Option<Text>) {
    match token.scalars.iter().position(|c| *c == '=' as u32) {
        Some(at) => {
            let units = token.text.units();
            let unit_at = crate::util::unit_offset(units, at);
            (
                &token.scalars[..at],
                Some(Text::from_units(units[unit_at + 1..].to_vec())),
            )
        }
        None => (&token.scalars[..], None),
    }
}

/// The text of a token from a scalar index on, for
/// the value glued behind a short option letter or
/// an '=' in a short cluster.
fn text_from_scalar(token: &Token, scalar_index: usize) -> Text {
    let units = token.text.units();
    let unit_at = crate::util::unit_offset(units, scalar_index);
    Text::from_units(units[unit_at..].to_vec())
}

/// The outcome of matching a long-option name
/// against a level's option set.
enum LongMatch {
    Exact(&'static str),
    Unique(&'static str),
    Ambiguous(Vec<&'static str>),
    None,
}

fn match_long(name: &[u32], options: &[&'static str]) -> LongMatch {
    let mut candidates: Vec<&'static str> = Vec::new();
    for option in options {
        let full = scalars_of_str(option);
        if full == name {
            return LongMatch::Exact(option);
        }
        if full.len() > name.len() && full[..name.len()] == *name {
            candidates.push(option);
        }
    }
    match candidates.len() {
        0 => LongMatch::None,
        1 => LongMatch::Unique(candidates[0]),
        _ => LongMatch::Ambiguous(candidates),
    }
}

enum ParseOutcome {
    Run(&'static str, Box<SubParsed>),
    Exit(i32),
}

/// Write bytes to stdout under its lock, reporting
/// only whether the write and its flush landed.
fn write_stdout(bytes: &[u8]) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(bytes).and_then(|_| handle.flush())
}

/// Write bytes to stderr under its lock, reporting
/// only whether the write and its flush landed.
fn write_stderr(bytes: &[u8]) -> Result<(), std::io::Error> {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    handle.write_all(bytes).and_then(|_| handle.flush())
}

/// The "error, ..." line's destination: stderr, or
/// stdout when stderr was closed at launch (the
/// Python backs a missing standard stream with the
/// surviving one, so its reports migrate there). A
/// failed write is dropped at this seam; the caller
/// keeps the exit its own rule assigns.
fn report_error(line: &str) {
    let mut bytes = line.as_bytes().to_vec();
    bytes.push(b'\n');
    if crate::util::closed_at_launch(2) {
        let _ = write_stdout(&bytes);
    } else {
        let _ = write_stderr(&bytes);
    }
}

fn parse_error(usage: &str, prog: &str, message: &str) -> ParseOutcome {
    let text = format!("{usage}\n{prog}: error: {message}\n");
    if crate::util::closed_at_launch(2) {
        // With stderr closed at launch the Python's
        // usage line migrates to stdout and its
        // error line is lost in the fallback stream;
        // the exit stays 2.
        let _ = write_stdout(format!("{usage}\n").as_bytes());
        return ParseOutcome::Exit(2);
    }
    match write_stderr(text.as_bytes()) {
        Ok(()) => ParseOutcome::Exit(2),
        // A usage that cannot be delivered collapses
        // to 1 where the Python's shutdown flush
        // failure gives 120 (the flush class, named
        // in the dossier).
        Err(_) => ParseOutcome::Exit(1),
    }
}

/// Print a help text the guarded way: a broken
/// stream is a caught write error with exit 1,
/// never a panic (the shutdown flush class, named
/// in the dossier). With stdout closed at launch
/// the help migrates to stderr and still exits 0,
/// as the Python's does.
fn print_help(text: &str) -> ParseOutcome {
    if crate::util::closed_at_launch(1) {
        return match write_stderr(text.as_bytes()) {
            Ok(()) => ParseOutcome::Exit(0),
            Err(_) => ParseOutcome::Exit(1),
        };
    }
    match write_stdout(text.as_bytes()) {
        Ok(()) => ParseOutcome::Exit(0),
        Err(error) => {
            report_error(&format!("error, {}", os_error_text(&error)));
            ParseOutcome::Exit(1)
        }
    }
}

struct SubParsed {
    positionals: Vec<Text>,
    values: Vec<(&'static str, Text)>,
}

impl SubParsed {
    fn get_text(&self, canonical: &str) -> Option<Text> {
        self.values
            .iter()
            .rev()
            .find_map(|(name, value)| (*name == canonical).then(|| value.clone()))
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OptKind {
    Help,
    Value,
}

struct OptSpec {
    canonical: &'static str,
    label: &'static str,
    kind: OptKind,
    short: Option<char>,
}

struct SubSpec {
    usage: &'static str,
    prog: &'static str,
    help: &'static str,
    positionals: &'static [&'static str],
    options: &'static [OptSpec],
    /// The required arguments in definition order:
    /// (label, is_positional).
    required: &'static [(&'static str, bool)],
}

const HELP_OPT: OptSpec = OptSpec {
    canonical: "--help",
    label: "-h/--help",
    kind: OptKind::Help,
    short: Some('h'),
};

const OUTPUT_OPT: OptSpec = OptSpec {
    canonical: "--output",
    label: "-o/--output",
    kind: OptKind::Value,
    short: Some('o'),
};

const PROPOSE_SPEC: SubSpec = SubSpec {
    usage: PROPOSE_USAGE,
    prog: "bluepencil propose",
    help: PROPOSE_HELP,
    positionals: &["rules", "observations"],
    options: &[HELP_OPT, OUTPUT_OPT],
    required: &[("rules", true), ("observations", true)],
};

const DIFF_SPEC: SubSpec = SubSpec {
    usage: DIFF_USAGE,
    prog: "bluepencil diff",
    help: DIFF_HELP,
    positionals: &["rules", "proposals"],
    options: &[HELP_OPT, OUTPUT_OPT],
    required: &[("rules", true), ("proposals", true)],
};

fn spec_for(command: &str) -> &'static SubSpec {
    match command {
        "propose" => &PROPOSE_SPEC,
        _ => &DIFF_SPEC,
    }
}

/// Take the value for an option: a glued form wins,
/// else the next token, which an option-shaped
/// token can never be. On success `index` still
/// points at the option token; the caller advances
/// past both.
fn take_value(
    spec: &SubSpec,
    tokens: &[Token],
    index: &mut usize,
    glued: Option<Text>,
    opt: &OptSpec,
) -> Result<Text, ParseOutcome> {
    if let Some(value) = glued {
        return Ok(value);
    }
    match tokens.get(*index + 1) {
        Some(next) if !option_shaped(next) => {
            *index += 1;
            Ok(next.text.clone())
        }
        _ => Err(parse_error(
            spec.usage,
            spec.prog,
            &format!("argument {}: expected one argument", opt.label),
        )),
    }
}

/// A short cluster token's outcome.
enum ClusterFlow {
    /// The token is an extra.
    Extra,
    /// The token was consumed with nothing recorded.
    Skip,
    /// Values were recorded from the cluster.
    Recorded,
    /// The parse ends here.
    Exit(ParseOutcome),
}

/// Read a short cluster. A first letter the command
/// does not know makes the whole token an extra.
/// Help fires when the token ends, unless a scan
/// error in it fired first; a value letter takes
/// the rest of the cluster as its glued value, or
/// the next token when the cluster ends there.
fn short_cluster(
    spec: &SubSpec,
    tokens: &[Token],
    index: &mut usize,
    values: &mut Vec<(&'static str, Text)>,
) -> ClusterFlow {
    let token = &tokens[*index];
    let cluster = &token.scalars[1..];
    let first = cluster[0] as u8 as char;
    let known = spec.options.iter().any(|opt| opt.short == Some(first));
    if !known {
        return ClusterFlow::Extra;
    }
    let mut help_pending = false;
    let mut recorded = false;
    let mut at = 0usize;
    while at < cluster.len() {
        let letter = cluster[at] as u8 as char;
        if letter == 'h' {
            if cluster.get(at + 1) == Some(&('=' as u32)) {
                let value = text_from_scalar(token, at + 3);
                return ClusterFlow::Exit(parse_error(
                    spec.usage,
                    spec.prog,
                    &format!(
                        "argument -h/--help: ignored explicit argument {}",
                        repr_text(&value)
                    ),
                ));
            }
            help_pending = true;
            at += 1;
            continue;
        }
        let opt = spec
            .options
            .iter()
            .find(|opt| opt.short == Some(letter) && opt.kind != OptKind::Help);
        match opt {
            Some(opt) => {
                let rest = &cluster[at + 1..];
                let glued = if rest.is_empty() {
                    None
                } else if rest[0] == '=' as u32 {
                    Some(text_from_scalar(token, at + 3))
                } else {
                    Some(text_from_scalar(token, at + 2))
                };
                match take_value(spec, tokens, index, glued, opt) {
                    Ok(value) => {
                        values.push((opt.canonical, value));
                        recorded = true;
                    }
                    Err(outcome) => return ClusterFlow::Exit(outcome),
                }
                // A value letter ends the cluster.
                at = cluster.len();
            }
            None => {
                // A letter the command does not know,
                // after known letters: where a help
                // letter already stands in the cluster
                // the help fires (argparse takes the
                // help action as the cluster is read);
                // otherwise argparse reads the whole
                // token as unrecognized.
                if help_pending {
                    return ClusterFlow::Exit(print_help(spec.help));
                }
                return ClusterFlow::Extra;
            }
        }
    }
    if help_pending {
        return ClusterFlow::Exit(print_help(spec.help));
    }
    if recorded {
        return ClusterFlow::Recorded;
    }
    ClusterFlow::Skip
}

fn parse_sub(command: &'static str, tokens: &[Token], top_extras: &[Text]) -> ParseOutcome {
    let spec = spec_for(command);
    let candidates: Vec<&'static str> = spec.options.iter().map(|opt| opt.canonical).collect();
    let mut positionals: Vec<Text> = Vec::new();
    let mut extras: Vec<Text> = top_extras.to_vec();
    let mut values: Vec<(&'static str, Text)> = Vec::new();
    let mut separated = false;
    let mut index = 0usize;

    fn push_positional(
        token: &Text,
        spec: &SubSpec,
        positionals: &mut Vec<Text>,
        extras: &mut Vec<Text>,
    ) {
        if positionals.len() < spec.positionals.len() {
            positionals.push(token.clone());
        } else {
            extras.push(token.clone());
        }
    }

    while index < tokens.len() {
        let token = &tokens[index];
        if separated {
            push_positional(&token.text, spec, &mut positionals, &mut extras);
            index += 1;
            continue;
        }
        if token.scalars == scalars_of_str("--") {
            // The separator is consumed wherever it
            // stands; what follows it is positional,
            // and overflows to the extras on its own.
            separated = true;
            index += 1;
            continue;
        }
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
        {
            let (name, glued) = split_long(token);
            let chosen = match match_long(name, &candidates) {
                LongMatch::Exact(option) | LongMatch::Unique(option) => option,
                LongMatch::Ambiguous(matches) => {
                    return parse_error(
                        spec.usage,
                        spec.prog,
                        &format!(
                            "ambiguous option: {} could match {}",
                            token.rendered(),
                            matches.join(", ")
                        ),
                    );
                }
                LongMatch::None => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            };
            let opt = spec
                .options
                .iter()
                .find(|opt| opt.canonical == chosen)
                .expect("a spec option");
            if opt.kind == OptKind::Help {
                if let Some(value) = glued {
                    return parse_error(
                        spec.usage,
                        spec.prog,
                        &format!(
                            "argument -h/--help: ignored explicit argument {}",
                            repr_text(&value)
                        ),
                    );
                }
                return print_help(spec.help);
            }
            match take_value(spec, tokens, &mut index, glued, opt) {
                Ok(value) => values.push((opt.canonical, value)),
                Err(outcome) => return outcome,
            }
            index += 1;
            continue;
        }
        if option_shaped(token) {
            match short_cluster(spec, tokens, &mut index, &mut values) {
                ClusterFlow::Extra => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
                ClusterFlow::Skip | ClusterFlow::Recorded => {
                    index += 1;
                    continue;
                }
                ClusterFlow::Exit(outcome) => return outcome,
            }
        }
        push_positional(&token.text, spec, &mut positionals, &mut extras);
        index += 1;
    }

    let mut missing: Vec<&str> = Vec::new();
    for (label, is_positional) in spec.required {
        if *is_positional {
            let at = spec
                .positionals
                .iter()
                .position(|name| name == label)
                .expect("a spec positional");
            if positionals.len() <= at {
                missing.push(label);
            }
        } else if !values.iter().any(|(name, _)| name == label) {
            missing.push(label);
        }
    }
    if !missing.is_empty() {
        return parse_error(
            spec.usage,
            spec.prog,
            &format!(
                "the following arguments are required: {}",
                missing.join(", ")
            ),
        );
    }
    if !extras.is_empty() {
        let rendered: Vec<String> = extras.iter().map(|t| stderr_render(t.units())).collect();
        return parse_error(
            TOP_USAGE,
            "bluepencil",
            &format!("unrecognized arguments: {}", rendered.join(" ")),
        );
    }
    ParseOutcome::Run(
        command,
        Box::new(SubParsed {
            positionals,
            values,
        }),
    )
}

const COMMANDS: [&str; 2] = ["propose", "diff"];

/// Parse the top level: help, the command word, and
/// the held extras an unknown token becomes.
fn parse_top(tokens: &[Token]) -> ParseOutcome {
    let mut extras: Vec<Text> = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.scalars.len() >= 2
            && token.scalars[0] == '-' as u32
            && token.scalars[1] == '-' as u32
            && token.scalars != scalars_of_str("--")
        {
            let (name, glued) = split_long(token);
            match match_long(name, &["--help"]) {
                LongMatch::Exact("--help") | LongMatch::Unique("--help") => {
                    if let Some(value) = glued {
                        return parse_error(
                            TOP_USAGE,
                            "bluepencil",
                            &format!(
                                "argument -h/--help: ignored explicit argument {}",
                                repr_text(&value)
                            ),
                        );
                    }
                    return print_help(TOP_HELP);
                }
                _ => {
                    extras.push(token.text.clone());
                    index += 1;
                    continue;
                }
            }
        }
        if token.scalars == scalars_of_str("--") || !option_shaped(token) {
            if token.scalars == scalars_of_str("--") && index == tokens.len() - 1 {
                break;
            }
            for command in COMMANDS {
                if token.text.units() == Text::from_str(command).units() {
                    let command: &'static str = command;
                    return parse_sub(command, &tokens[index + 1..], &extras);
                }
            }
            return parse_error(
                TOP_USAGE,
                "bluepencil",
                &format!(
                    "argument command: invalid choice: {} (choose from 'propose', 'diff')",
                    repr_text(&token.text)
                ),
            );
        }
        if token.scalars[1] == 'h' as u32 {
            if token.scalars.get(2) == Some(&('=' as u32)) {
                let value = text_from_scalar(token, 3);
                return parse_error(
                    TOP_USAGE,
                    "bluepencil",
                    &format!(
                        "argument -h/--help: ignored explicit argument {}",
                        repr_text(&value)
                    ),
                );
            }
            return print_help(TOP_HELP);
        }
        extras.push(token.text.clone());
        index += 1;
    }
    parse_error(
        TOP_USAGE,
        "bluepencil",
        "the following arguments are required: command",
    )
}

/// The Python's str of a path argument: pathlib's
/// normalization, redundant separators and dot
/// components dropped, a doubled leading separator
/// kept, and an empty path read as the dot.
fn normalize_path(units: &[u16]) -> Vec<u16> {
    let scalars = crate::util::scalars_of(units);
    let leading = scalars.iter().take_while(|scalar| **scalar == 0x2F).count();
    let mut parts: Vec<&[u32]> = Vec::new();
    for part in scalars.split(|scalar| *scalar == 0x2F) {
        if part.is_empty() || part == [0x2E] {
            continue;
        }
        parts.push(part);
    }
    if parts.is_empty() {
        // BP-35: a componentless spelling keeps its
        // anchor: a leading pair stands as the
        // anchor, a longer run collapses to one, and
        // no separator at all reads as the dot.
        let anchor: &[u32] = if leading == 2 {
            &[0x2F, 0x2F]
        } else if leading > 0 {
            &[0x2F]
        } else {
            &[0x2E]
        };
        return anchor.iter().map(|unit| *unit as u16).collect();
    }
    let mut out: Vec<u32> = Vec::new();
    if leading == 2 {
        out.extend_from_slice(&[0x2F, 0x2F]);
    } else if leading > 0 {
        out.push(0x2F);
    }
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(0x2F);
        }
        out.extend_from_slice(part);
    }
    crate::util::text_from_scalars(&out).units().to_vec()
}

/// A path resolved the way pathlib's resolve
/// reads: the longest existing prefix is
/// canonicalized, the rest is appended and then
/// normalized lexically, a dot component dropping
/// and a dot-dot popping the component before it,
/// whatever kind of component that is. Resolution
/// fails outright (None) where the interpreter's
/// resolve raises: for a relative spelling whose
/// cwd cannot be read (BP-33), and where the
/// finished path passes
/// through a symlink loop, which the output guard
/// counts as naming an input (BP-16a). The loop is
/// judged on the finished path, not on the
/// spellings the peel tries along the way: a loop
/// component a following dot-dot cancels never
/// survives into the finished path, and the
/// interpreter resolves those spellings cleanly
/// (BP-27). A dangling final link hands over its
/// target spelling before any of this (see
/// dangling_target), and a chain still dangling
/// at the kernel's link bound fails outright,
/// like a loop (BP-27a).
/// The kernel's symlink-loop errno on this
/// platform family.
const ELOOP: i32 = 40;

/// The rest of pathlib's exists() ignore set
/// (BP-27a): the errnos a stat may fail with that
/// the interpreter reads as absence. A stat
/// failing any other way is raised by exists(),
/// and the output guard counts the raise as
/// naming an input.
const ENOENT: i32 = 2;
const ENOTDIR: i32 = 20;
const EBADF: i32 = 9;

fn resolve_units(units: &[u16]) -> Option<Vec<u16>> {
    let raw = crate::util::os_path_from_units(units);
    let spelled = if raw.is_absolute() {
        raw
    } else {
        // BP-33: from a deleted working directory
        // the interpreter's resolve raises from
        // getcwd for a relative spelling, so a
        // failed cwd read fails the resolution
        // outright, and the guard counts the
        // failure as naming an input. An absolute
        // spelling never consults the cwd.
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(raw),
            Err(_) => return None,
        }
    };
    let absolute = match dangling_target(&spelled) {
        Dangling::Spelled => spelled,
        Dangling::Subject(subject) => subject,
        Dangling::Unresolvable => return None,
    };
    let mut prefix = absolute.clone();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let resolved = loop {
        match std::fs::canonicalize(&prefix) {
            Ok(mut resolved) => {
                for part in tail.iter().rev() {
                    resolved.push(part);
                }
                break resolved;
            }
            Err(_) => {
                let last = prefix.components().next_back();
                match last {
                    Some(component) => {
                        tail.push(component.as_os_str().to_os_string());
                        if !prefix.pop() {
                            break prefix;
                        }
                    }
                    None => break prefix,
                }
            }
        }
    };
    let bytes = crate::util::os_bytes(resolved.as_os_str());
    let units = fsdecode_units(&bytes);
    let scalars = crate::util::scalars_of(&units);
    let mut parts: Vec<&[u32]> = Vec::new();
    for part in scalars.split(|scalar| *scalar == 0x2F) {
        if part.is_empty() || part == [0x2E] {
            continue;
        }
        if part == [0x2E, 0x2E] {
            parts.pop();
            continue;
        }
        parts.push(part);
    }
    let mut out: Vec<u32> = vec![0x2F];
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(0x2F);
        }
        out.extend_from_slice(part);
    }
    let finished = crate::util::text_from_scalars(&out).units().to_vec();
    if path_loops(&finished) {
        return None;
    }
    Some(finished)
}

/// How a spelled path's dangling final link hands
/// the resolution its subject (BP-27, BP-27a).
enum Dangling {
    /// No dangling final link: the resolution
    /// proceeds on the spelled path itself.
    Spelled,
    /// The resolution proceeds on this spelling:
    /// the target spelling of the dangling link,
    /// or of the last link of a dangling chain.
    Subject(std::path::PathBuf),
    /// The chain is still dangling at the kernel's
    /// link bound: the path counts as unresolvable.
    Unresolvable,
}

/// The kernel's bound on the symlink traversals
/// one resolution performs: a dangling chain
/// still unresolvable at the bound counts as
/// naming an input (BP-27a), as the interpreter's
/// resolve raises on the chain.
const MAX_SPLICES: u32 = 40;

/// The subject a dangling final link hands the
/// resolution (BP-27): where the spelled path's
/// final component is a symlink whose target
/// cannot be canonicalized (the target may not
/// exist as spelled, may pass through a missing
/// component, may sit behind an unsearchable
/// prefix, or may itself loop), realpath reads
/// the target spelling and proceeds on it
/// resolved against the link's parent, popping a
/// missing component in the spelling lexically,
/// so the spelling can land on an input the
/// spelled path never names. The peel alone would
/// peel the link's own name and lose the target
/// entirely. The splice iterates (BP-27a): where
/// the substituted spelling itself ends in a
/// dangling link, that link's target spelling is
/// spliced in turn, up to the kernel's link
/// bound. Anything else (no final link, or a
/// target that canonicalizes) leaves the
/// resolution on the spelled path, whose own
/// canonicalize follows the whole chain.
fn dangling_target(spelled: &std::path::Path) -> Dangling {
    let mut subject = spelled.to_path_buf();
    let mut splices = 0;
    while let Ok(meta) = std::fs::symlink_metadata(&subject) {
        if !meta.file_type().is_symlink() || std::fs::canonicalize(&subject).is_ok() {
            break;
        }
        if splices == MAX_SPLICES {
            return Dangling::Unresolvable;
        }
        let target = match std::fs::read_link(&subject) {
            Ok(target) => target,
            Err(_) => break,
        };
        let parent = match subject.parent() {
            Some(parent) => parent.to_path_buf(),
            None => break,
        };
        subject = parent.join(&target);
        splices += 1;
    }
    if subject.as_path() == spelled {
        Dangling::Spelled
    } else {
        Dangling::Subject(subject)
    }
}

/// True where the finished path passes through a
/// symlink loop: canonicalizing its own prefixes
/// one component at a time meets ELOOP. A prefix
/// that does not exist ends the walk, as the rest
/// of the path is a lexical tail no loop can hide
/// in; a prefix failing any other way (an
/// unsearchable directory, say) ends it too, the
/// peel above having treated it the same way.
fn path_loops(units: &[u16]) -> bool {
    let path = crate::util::os_path_from_units(units);
    let mut prefix = std::path::PathBuf::from("/");
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            prefix.push(name);
            match std::fs::canonicalize(&prefix) {
                Ok(_) => {}
                Err(error) if error.raw_os_error() == Some(ELOOP) => return true,
                Err(_) => break,
            }
        }
    }
    false
}

/// How a spelled path stands to a stat, in the
/// terms the oracle's guard asks the question
/// (BP-27a): pathlib's exists() reports a standing
/// path, reads a failure in its ignore set as
/// absence, and raises any other failure, an
/// unsearchable directory on the path (EACCES)
/// being the probed instance.
enum GuardStat {
    Exists,
    Absent,
    Failed,
}

fn guard_stat(path: &std::path::Path) -> GuardStat {
    match std::fs::metadata(path) {
        Ok(_) => GuardStat::Exists,
        Err(error) => match error.raw_os_error() {
            Some(ENOENT) | Some(ENOTDIR) | Some(EBADF) | Some(ELOOP) => GuardStat::Absent,
            _ => GuardStat::Failed,
        },
    }
}

/// BP-16a: true when the output names an input, by
/// resolved path or by file identity. A path that
/// cannot be resolved is treated as naming one.
/// The guard mirrors the oracle's call structure:
/// one guard covers resolve() and exists()
/// together, so a stat the interpreter raises on,
/// the output's or an input's, counts as naming
/// an input (BP-27a).
fn writes_to_input(out: &[u16], inputs: &[Vec<u16>]) -> bool {
    let resolved_out = match resolve_units(out) {
        Some(resolved) => resolved,
        None => return true,
    };
    for input in inputs {
        match resolve_units(input) {
            Some(resolved) => {
                if resolved == resolved_out {
                    return true;
                }
            }
            None => return true,
        }
    }
    let out_path = crate::util::os_path_from_units(out);
    match guard_stat(&out_path) {
        GuardStat::Failed => return true,
        GuardStat::Absent => return false,
        GuardStat::Exists => {}
    }
    for input in inputs {
        let input_path = crate::util::os_path_from_units(input);
        match guard_stat(&input_path) {
            GuardStat::Failed => return true,
            GuardStat::Absent => continue,
            GuardStat::Exists => {}
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // os.path.samefile stats both paths and
            // raises on a failure, which the guard
            // counts as naming an input.
            match (std::fs::metadata(&out_path), std::fs::metadata(&input_path)) {
                (Ok(left), Ok(right)) => {
                    if left.dev() == right.dev() && left.ino() == right.ino() {
                        return true;
                    }
                }
                _ => return true,
            }
        }
    }
    false
}

static TEMP_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// posixpath.normpath over an absolute spelling:
/// empty and dot components drop, a dotdot pops
/// the spelled component before it and a pop at
/// the root drops, and a leading separator pair
/// stands as the anchor where one or three or
/// more collapse to one.
fn normpath_units(units: &[u16]) -> Vec<u16> {
    let scalars = crate::util::scalars_of(units);
    let leading = scalars.iter().take_while(|scalar| **scalar == 0x2F).count();
    let mut parts: Vec<&[u32]> = Vec::new();
    for part in scalars.split(|scalar| *scalar == 0x2F) {
        if part.is_empty() || part == [0x2E] {
            continue;
        }
        if part == [0x2E, 0x2E] {
            parts.pop();
            continue;
        }
        parts.push(part);
    }
    let mut out: Vec<u32> = Vec::new();
    if leading == 2 {
        out.extend_from_slice(&[0x2F, 0x2F]);
    } else {
        out.push(0x2F);
    }
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push(0x2F);
        }
        out.extend_from_slice(part);
    }
    crate::util::text_from_scalars(&out).units().to_vec()
}

/// The parent a temp is created under, absolutized
/// the way mkstemp absolutizes it (BP-41): the
/// interpreter's _mkstemp_inner opens with dir =
/// os.path.abspath(dir), a cwd join when the
/// parent is relative and then the normpath law,
/// with no link resolution anywhere in that step.
/// An absolute parent never consults the cwd. A
/// failed cwd read on a relative parent fails the
/// step, as the interpreter's abspath raises
/// there; the guard's own BP-33 refusal makes
/// that arm unreachable for a named output, and
/// it stands as the raise's mirror.
fn lexical_abspath(parent: &[u16]) -> Option<Vec<u16>> {
    let absolute = if parent.first() == Some(&0x2F) {
        parent.to_vec()
    } else {
        let cwd = match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => return None,
        };
        let mut joined = fsdecode_units(&crate::util::os_bytes(cwd.as_os_str()));
        if joined.last() != Some(&0x2F) {
            joined.push(0x2F);
        }
        joined.extend_from_slice(parent);
        joined
    };
    Some(normpath_units(&absolute))
}

/// The mkstemp-shaped write: a sibling temp with
/// the target's name and a dot for a prefix, no
/// suffix, mode 0600, replaced onto the target,
/// and unlinked on any failure. The temp is
/// created under the parent's lexically
/// absolutized form (BP-41), not under the
/// parent as spelled: a symlink component the
/// spelling pops with a following dotdot is
/// never traversed for the temp, exactly as
/// mkstemp's abspath never traverses it. The
/// rename still targets the path itself. The
/// candidate is the name's own bytes, a dot,
/// and exactly eight drawn characters (BP-47):
/// the lossy name and the decimal pid, counter,
/// and attempt suffix it replaces stood
/// convicted on all three arms in Round 7
/// (BP-44, BP-45, BP-46). The draw's terms
/// stand in draw_suffix; Round 8 adds the
/// entropy term the frozen-clock conviction
/// is owed to (BP-54, BP-55).
/// Sixteen bytes of operating system
/// randomness (BP-55), drawn from
/// /dev/urandom, the kernel's randomness
/// interface, through the standard library's
/// file read. Any failure, the open or a
/// short or failed read, yields None. The
/// crate's non-unix arm has no operating
/// system randomness source to read and
/// yields None on every call, so a build
/// there refuses the write rather than
/// drawing computably.
#[cfg(unix)]
fn os_entropy() -> Option<[u8; 16]> {
    use std::io::Read;
    let mut bytes = [0u8; 16];
    let mut file = std::fs::File::open("/dev/urandom").ok()?;
    file.read_exact(&mut bytes).ok()?;
    Some(bytes)
}

#[cfg(not(unix))]
fn os_entropy() -> Option<[u8; 16]> {
    None
}

/// One candidate's eight drawn characters
/// (BP-47, BP-55): the first four bytes of a
/// SHA-256 digest rendered as eight lowercase
/// hex characters. A clock that cannot be
/// read, or reads before the epoch, yields
/// None (BP-47a), and an entropy source that
/// cannot be read yields None (BP-55); the
/// caller refuses the write on None, and the
/// draw never falls back to a computable mix.
fn draw_suffix(counter: u32, attempt: u32, dir_bytes: &[u8], name_bytes: &[u8]) -> Option<Vec<u8>> {
    let nanos = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => return None,
    };
    let entropy = os_entropy()?;
    Some(suffix_from_terms(
        nanos, counter, attempt, dir_bytes, name_bytes, &entropy,
    ))
}

/// The suffix the terms decide, factored out
/// of draw_suffix so the tests can pin the
/// mix term by term: the hash input is the
/// process id, the nanoseconds, the counter,
/// the attempt, the directory's bytes, the
/// name's bytes, and the entropy, in that
/// order, each integer little-endian. The
/// entropy term is what keeps the candidate
/// set from being a public function of inputs
/// an observer can know or fix (BP-55): the
/// earlier terms alone stand still under a
/// frozen and known clock.
fn suffix_from_terms(
    nanos: u128,
    counter: u32,
    attempt: u32,
    dir_bytes: &[u8],
    name_bytes: &[u8],
    entropy: &[u8; 16],
) -> Vec<u8> {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(nanos.to_le_bytes());
    hasher.update(counter.to_le_bytes());
    hasher.update(attempt.to_le_bytes());
    hasher.update(dir_bytes);
    hasher.update(name_bytes);
    hasher.update(entropy);
    let digest = hasher.finalize();
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut suffix = Vec::with_capacity(8);
    for byte in &digest[..4] {
        suffix.push(HEX[usize::from(byte >> 4)]);
        suffix.push(HEX[usize::from(byte & 0xF)]);
    }
    suffix
}

fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> Result<(), ()> {
    let dir = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    let parent_units = fsdecode_units(&crate::util::os_bytes(dir.as_os_str()));
    let dir = match lexical_abspath(&parent_units) {
        Some(units) => crate::util::os_path_from_units(&units),
        None => return Err(()),
    };
    let name_bytes = path
        .file_name()
        .map(crate::util::os_bytes)
        .unwrap_or_default();
    let dir_bytes = crate::util::os_bytes(dir.as_os_str());
    let mut attempt = 0u32;
    let tmp = loop {
        // BP-47, BP-55: the candidate's drawn
        // characters come from draw_suffix over
        // the process id, the wall-clock time in
        // nanoseconds, the write counter, the
        // attempt, the temp directory's bytes,
        // the name's bytes, and sixteen bytes of
        // operating system entropy. A draw that
        // yields None refuses the write.
        let counter = TEMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let suffix = match draw_suffix(counter, attempt, &dir_bytes, &name_bytes) {
            Some(suffix) => suffix,
            None => return Err(()),
        };
        let mut candidate_bytes = name_bytes.clone();
        candidate_bytes.push(b'.');
        candidate_bytes.extend_from_slice(&suffix);
        let candidate = dir.join(crate::util::os_path_from_units(&fsdecode_units(
            &candidate_bytes,
        )));
        #[cfg(unix)]
        let opener = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true).mode(0o600);
            options
        };
        #[cfg(not(unix))]
        let opener = {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            options
        };
        match opener.open(&candidate) {
            Ok(mut file) => match file.write_all(bytes).and_then(|_| file.flush()) {
                Ok(()) => break candidate,
                Err(_) => {
                    let _ = std::fs::remove_file(&candidate);
                    return Err(());
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                attempt += 1;
                if attempt > 64 {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }
    };
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            Err(())
        }
    }
}

/// The Python's _emit: write the rendered bytes to
/// stdout or atomically to the output, and on a
/// failure name it with the output as typed.
fn emit(
    bytes_result: Result<Vec<u8>, crate::emit::EmitError>,
    out: Option<&Text>,
    noun: &str,
) -> i32 {
    let out_path = out.map(|text| crate::util::os_path_from_units(text.units()));
    let bytes = match bytes_result {
        Ok(bytes) => bytes,
        Err(_) => {
            return cannot_write(out, noun);
        }
    };
    match out_path {
        None => {
            if crate::util::closed_at_launch(1) {
                return 1;
            }
            match write_stdout(&bytes) {
                Ok(()) => 0,
                Err(_) => cannot_write(None, noun),
            }
        }
        Some(path) => match write_atomically(&path, &bytes) {
            Ok(()) => 0,
            Err(()) => cannot_write(out, noun),
        },
    }
}

fn cannot_write(out: Option<&Text>, noun: &str) -> i32 {
    match out {
        // The interpreter formats the output as a
        // Path, so the name in this message is the
        // pathlib-normalized form, not the raw
        // argument.
        Some(text) => report_error(&format!(
            "error, cannot write the {noun} file, {}",
            stderr_render(&normalize_path(text.units()))
        )),
        None => report_error(&format!("error, cannot write the {noun} to stdout")),
    }
    1
}

/// A module failure at the CLI seam: a named
/// refusal after "error, ", and the crash class as
/// a bare exit 1.
fn report_failure(failure: Failure) -> i32 {
    match failure {
        Failure::Named(message) => {
            report_error(&format!("error, {}", message.render_stderr()));
            1
        }
        Failure::Crash => 1,
    }
}

fn named_error(message: &str) -> i32 {
    report_error(&format!("error, {message}"));
    1
}

fn path_error(prefix: &str, units: &[u16]) -> i32 {
    report_error(&format!("error, {prefix}, {}", stderr_render(units)));
    1
}

/// The preamble both commands share: the output
/// value extracted, the output guard run over both
/// inputs, the existence checks in order, and the
/// rules file read and decoded. The commands
/// differ only in the second input's noun and in
/// what they load from it afterwards.
struct Preamble {
    output: Option<Text>,
    second_arg: Vec<u16>,
    rules_bytes: Vec<u8>,
    rules_text: Text,
}

fn preamble(parsed: &SubParsed, second_noun: &str) -> Result<Preamble, i32> {
    let rules_arg = normalize_path(parsed.positionals[0].units());
    let second_arg = normalize_path(parsed.positionals[1].units());
    // BP-34: the output spelling is constructed
    // once, as the interpreter builds one Path
    // from the argument, and the constructed
    // spelling is what the guard resolves and
    // stats and what the write replaces.
    let output = parsed
        .get_text("--output")
        .filter(|text| !text.units().is_empty())
        .map(|text| Text::from_units(normalize_path(text.units())));
    if let Some(out) = &output {
        if writes_to_input(out.units(), &[rules_arg.clone(), second_arg.clone()]) {
            return Err(named_error("bluepencil never writes to its inputs"));
        }
    }
    let rules_path = crate::util::os_path_from_units(&rules_arg);
    let second_path = crate::util::os_path_from_units(&second_arg);
    if !rules_path.is_file() {
        return Err(path_error("no such rules file", &rules_arg));
    }
    if !second_path.is_file() {
        return Err(path_error(
            &format!("no such {second_noun} file"),
            &second_arg,
        ));
    }
    let rules_bytes = match std::fs::read(&rules_path) {
        Ok(bytes) => bytes,
        Err(_) => return Err(path_error("cannot read the rules file", &rules_arg)),
    };
    let rules_text = match decode_utf8_sig(&rules_bytes) {
        Some(text) => Text::from_str(&text),
        None => return Err(path_error("the rules file is not utf-8", &rules_arg)),
    };
    Ok(Preamble {
        output,
        second_arg,
        rules_bytes,
        rules_text,
    })
}

fn propose(parsed: &SubParsed) -> i32 {
    let pre = match preamble(parsed, "observations") {
        Ok(pre) => pre,
        Err(code) => return code,
    };
    let observations_path = crate::util::os_path_from_units(&pre.second_arg);
    let (rules, warnings) = match parse_rules(&pre.rules_text) {
        Ok(pair) => pair,
        Err(failure) => return report_failure(failure),
    };
    let (observations, rewrites) = match load_observations(&observations_path) {
        Ok(pair) => pair,
        Err(failure) => return report_failure(failure),
    };
    let document = match crate::proposals::build_document(
        &pre.rules_bytes,
        &rules,
        &observations,
        &rewrites,
        &warnings,
    ) {
        Ok(document) => document,
        Err(failure) => return report_failure(failure),
    };
    let mode = if pre.output.is_some() {
        Mode::Strict
    } else {
        Mode::Stdout
    };
    emit(
        dumps_document(&document, mode),
        pre.output.as_ref(),
        "proposal",
    )
}

fn diff(parsed: &SubParsed) -> i32 {
    let pre = match preamble(parsed, "proposals") {
        Ok(pre) => pre,
        Err(code) => return code,
    };
    let rules_bytes = &pre.rules_bytes;
    let rules_text = &pre.rules_text;
    let proposals_path = crate::util::os_path_from_units(&pre.second_arg);
    let proposals_bytes = match std::fs::read(&proposals_path) {
        Ok(bytes) => bytes,
        Err(_) => return path_error("cannot read the proposals file", &pre.second_arg),
    };
    let document = match crate::diff::load_proposals(&proposals_bytes) {
        Ok(document) => document,
        Err(failure) => return report_failure(failure),
    };
    if let Err(failure) = crate::diff::check_target(rules_bytes, &document) {
        return report_failure(failure);
    }
    let (rules, _warnings) = match parse_rules(rules_text) {
        Ok(pair) => pair,
        Err(failure) => return report_failure(failure),
    };
    let lines = crate::util::py_splitlines(rules_text.units());
    let text = match crate::diff::render_diff(&lines, &rules, &document) {
        Ok(text) => text,
        Err(failure) => return report_failure(failure),
    };
    let mode = if pre.output.is_some() {
        Mode::Strict
    } else {
        Mode::Stdout
    };
    emit(encode_text(&text, mode), pre.output.as_ref(), "diff")
}

fn os_error_text(err: &std::io::Error) -> String {
    match err.raw_os_error() {
        Some(_) => format!(
            "[Errno {}] {}",
            err.raw_os_error().unwrap_or(0),
            crate::util::strerror_text(err)
        ),
        None => format!("{err}"),
    }
}

/// The command line entry.
pub fn run_cli() -> i32 {
    let tokens: Vec<Token> = std::env::args_os()
        .skip(1)
        .map(|arg| Token::from_units(fsdecode_units(&crate::util::os_bytes(arg.as_os_str()))))
        .collect();
    match parse_top(&tokens) {
        ParseOutcome::Run(command, parsed) => match command {
            "propose" => propose(&parsed),
            _ => diff(&parsed),
        },
        ParseOutcome::Exit(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(
        nanos: u128,
        counter: u32,
        attempt: u32,
        dir_bytes: &[u8],
        name_bytes: &[u8],
        entropy: &[u8; 16],
    ) -> String {
        String::from_utf8(suffix_from_terms(
            nanos, counter, attempt, dir_bytes, name_bytes, entropy,
        ))
        .expect("the suffix is ascii")
    }

    #[test]
    fn the_entropy_term_decides_the_suffix() {
        // With every other term fixed, the
        // entropy alone moves the suffix
        // (BP-55): a mix that dropped or
        // ignored the term fails here, and
        // the same terms, entropy included,
        // decide the same suffix, so the
        // entropy is an input to the mix and
        // not a hidden state.
        let nanos = 1_700_000_000_123_456_789u128;
        let a = render(nanos, 3, 3, b"/tmp/bp8", b"out.json", &[7u8; 16]);
        let b = render(nanos, 3, 3, b"/tmp/bp8", b"out.json", &[8u8; 16]);
        assert_ne!(a, b);
        assert_eq!(a, render(nanos, 3, 3, b"/tmp/bp8", b"out.json", &[7u8; 16]));
    }

    #[test]
    fn the_suffix_matches_an_independent_derivation() {
        // The expected suffix is derived here
        // from the contracted input layout
        // (BP-47, BP-55), term by term, rather
        // than trusted from the construction
        // under test.
        use sha2::Digest;
        let nanos = 1_700_000_000_123_456_789u128;
        let entropy = [42u8; 16];
        let mut hasher = sha2::Sha256::new();
        hasher.update(std::process::id().to_le_bytes());
        hasher.update(nanos.to_le_bytes());
        hasher.update(3u32.to_le_bytes());
        hasher.update(3u32.to_le_bytes());
        hasher.update(b"/tmp/bp8");
        hasher.update(b"out.json");
        hasher.update(entropy);
        let digest = hasher.finalize();
        let mut expected = String::new();
        for byte in &digest[..4] {
            expected.push_str(&format!("{byte:02x}"));
        }
        assert_eq!(expected.len(), 8);
        assert_eq!(
            render(nanos, 3, 3, b"/tmp/bp8", b"out.json", &entropy),
            expected
        );
    }

    #[test]
    fn a_live_draw_renders_eight_hex_characters_or_refuses() {
        // A live draw over fixed terms ends
        // one of the two ways the contract
        // allows (BP-55). Where the entropy
        // source reads, the suffix keeps its
        // contracted shape: eight lowercase
        // hexadecimal characters. Where it
        // cannot be read, and on every
        // non-unix build, whose entropy arm
        // yields None on every call, the draw
        // yields None and the caller refuses
        // the write, so the pin accepts the
        // refusal rather than panicking on
        // it. Two draws are not required to
        // differ: the suffix carries
        // thirty-two bits of the hash, so a
        // collision between two valid draws
        // breaks no term of the contract.
        for attempt in 0..=1 {
            if let Some(suffix) = draw_suffix(0, attempt, b"/tmp/bp8", b"out.json") {
                assert_eq!(suffix.len(), 8);
                assert!(suffix
                    .iter()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)));
            }
        }
    }
}
