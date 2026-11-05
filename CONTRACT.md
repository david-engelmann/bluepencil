# bluepencil Contract, stub

The full Contract is the first build act of this repo phase. This
stub records the obligations the setup already knows it owes. Each
obligation becomes executable before any feature code that depends
on it lands.

- BP-1. Inputs are rules files such as CLAUDE.md and AGENTS.md, plus session transcripts. Nothing else is judged.
- BP-2. Output includes per rule exercise and obedience counts, with the counting method stated.
- BP-3. Proposals come in three kinds only, delete, rewrite with the misreading quoted, or promote. No other kind ships.
- BP-4. Output is a diff for a human, never auto applied. Bluepencil judges the instructions, stickler judges the code.

## Round 1. Rules inventory and evidence joined proposals (BP-5 to BP-12)

Contract preparation only in this pass. The build opens
when this repo's turn in the line's build order arrives.

- BP-5. A rules file parses into an inventory: every
  bullet or numbered line under a heading is a rule,
  carrying an id by position, its section path, its exact
  text, and its line span. Prose paragraphs are context,
  never rules. The inventory covers the whole file or the
  parse refuses by name; a rule is never silently dropped.
  Test, tests/test_inventory.py.
- BP-6. Evidence enters as an observations file, a JSON
  document of observations, each naming a rule by its
  exact text, a kind (misreading, ignored, or followed),
  and a quote. An observation that matches no rule is
  reported as unmatched, never silently attached to a
  near miss. Matching is by exact text, so a rule edited
  since the observation was taken is a new rule.
  Test, tests/test_inventory.py.
- BP-7. The inventory joined with the observations
  carries, per rule, its exercise and obedience counts by
  kind, with the counting method stated in the output, as
  the stub's BP-2 requires. Test, tests/test_inventory.py.
- BP-8. A proposal file is a JSON document naming the
  rules file it targets by the SHA-256 of its bytes, and
  a list of proposals. Each proposal names one rule, one
  kind from the stub's three (delete, rewrite, promote),
  the rule's text as it stands, and the observations it
  rests on, quoted. A proposal resting on no observation
  is invalid, and the validator names it.
  Test, tests/test_proposals.py.
- BP-9. A rewrite proposal additionally quotes the
  misreading it answers, and its replacement text comes
  from the observation set the operator supplies, never
  invented by the tool. Round 1 generates no rewrite on
  its own; it validates and carries the ones supplied.
  Test, tests/test_proposals.py.
- BP-10. Delete generation is mechanical: a rule with at
  least two ignored observations and no followed
  observation earns a delete proposal citing them.
  Promote generation is mechanical: a rule in the second
  half of the file by rule order, with at least two
  followed observations, earns a promote proposal whose
  hunk moves it to the first section. Below either
  threshold, no proposal. Test, tests/test_proposals.py.
- BP-11. The proposal file is deterministic: the same
  rules file and observations produce byte-identical
  output, rules and proposals in file order.
  Test, tests/test_proposals.py.
- BP-12. Bluepencil never writes to the rules file it
  judges. Its only output is the proposal file, to the
  path named on the command line or to stdout, and a
  write failure is a named error with exit 1, in the
  family's discipline. An apply mode does not exist in
  Round 1. Test, tests/test_proposals.py.

## Round 1 amendment, parse clarifications (recorded before code)

- BP-5a. A rule line appearing before the first heading
  is a named refusal: it has no section path, and the
  inventory will not invent one. Lines inside fenced
  code blocks are never rules. An indented bullet or
  numbered line is a rule in its own right, in document
  order. A rule's text is its marker line plus any
  indented continuation lines, and its span covers them.
  Test, tests/test_inventory.py.
- BP-10a. Second half by rule order means the rule at
  zero-based index i of n rules qualifies when 2i is at
  least n. The first section is the section holding the
  first rule. A promote whose rule already sits in the
  first section is no proposal at all. Test,
  tests/test_proposals.py.

## Round 1 repair amendments (recorded after the three
reviews, before the repair code)

- BP-5b. A bullet or numbered marker with no text after
  it is a named refusal, an empty rule at its line: it is
  neither inventoried nor silently dropped.
- BP-5c. Lines inside an HTML comment block are context,
  never rules, headings, or fences. A setext heading (a
  prose line underlined with equals signs or dashes)
  opens a section like an ATX heading does. A fence
  closes only on its own character at at least its own
  length. A task list checkbox is syntax, stripped from
  a rule's text. If the file ends inside a fence, the
  parse completes and the proposal file carries a
  warning naming the fence's line.
- BP-8a. The proposal validator also refuses a proposal
  whose kind is outside the stub's three, and evidence
  that carries no quote.

## Round 2. The diff for a human (BP-13 to BP-17)

Round 1 shipped the JSON proposal file; the stub's
BP-4 also promises a diff for a human, and Round 1 did
not build one. Round 2 builds it. The diff renders
proposals for review; it applies nothing, and BP-4
and BP-12 stand unchanged.

- BP-13. The diff command takes a rules file and a
  proposal file. The proposal file must name the rules
  file by the SHA-256 of its bytes, as BP-8 requires,
  and must pass the proposal validator; a mismatch, a
  malformed proposal file, or an invalid one is a
  named refusal with exit 1 and nothing rendered. The
  rules file is parsed fresh, and a parse refusal is
  the inventory's own. Test, tests/test_diff.py.
- BP-14. Every proposal is cross-checked before
  anything renders: its rule id exists in the proposal
  file's inventory carrying the same text the proposal
  carries, and the freshly parsed rules file holds a
  rule at the same id with the same text and the same
  section. Any mismatch is a named refusal naming the
  proposal. The diff renders only what it can stand
  over. Test, tests/test_diff.py.
- BP-15. Rendering is per proposal, in proposal-file
  order. A block names the rule id, the kind, and the
  section path. A delete shows the rule's own file
  lines, its line span, each prefixed with "- ". A
  rewrite shows those lines and then the replacement
  prefixed with "+ ", the original line's marker
  carried onto the replacement's first line (a task
  list checkbox is syntax, BP-5c, and is not carried).
  A promote shows the rule's lines as a removal and
  its header names the destination section. Each block
  lists the proposal's evidence as context lines, kind
  and quote, with newlines in a quote flattened to
  spaces. The diff contains no rule and no file line
  the proposals do not name. Test, tests/test_diff.py.
- BP-16. The diff is text for a human, never an
  applier: the command writes only the diff, to stdout
  or to the path named by -o, never onto the rules
  file or the proposal file, refused by name; the
  write is atomic in the family's discipline and a
  write failure is a named error with exit 1. Exit 0
  when the diff renders, including an empty proposal
  list, whose diff is its header and no blocks. Test,
  tests/test_diff.py.
- BP-17. The same rules file and proposal file
  produce a byte-identical diff. The header states the
  target SHA-256 in full and the proposal counts by
  kind, in the order delete, rewrite, promote. Test,
  tests/test_diff.py.

## Round 2 post-review amendments, recorded before the repairs

- BP-13a. The malformed arm of BP-13 reaches every
  field BP-8's document definition names. Each proposal
  is an object carrying a rule id with no line break,
  the rule's text, a kind of the three, an evidence
  list whose items are objects with a kind of the three
  observation kinds and a string quote, and a hunk
  object of its kind's shape; a rewrite also carries a
  string replacement. Each inventory entry is an object
  carrying an id, the rule's text, its section, and its
  line span. The target is a string of 64 hex
  characters. Anything less is a named refusal, never
  a traceback. The first implementation shape-checked
  only the top level and the renderer indexed the rest;
  the reviews drove fifteen traceback shapes through
  it. Test, tests/test_diff.py.
- BP-14a. The cross-check extends to the hunk. A
  delete's hunk names the rule's section and text. A
  rewrite's hunk names the section, the rule's text as
  its old, and the replacement as its new. A promote's
  hunk names the rule's section as its source, the
  first section as its destination, and the rule's
  text. A hunk that disagrees is a named refusal
  naming the proposal; the first implementation
  rendered a promote's destination on trust.
  Test, tests/test_diff.py.
- BP-15a. Nothing a proposal file carries can mint a
  new line in the diff. Every interpolated value is
  validated against a closed set (the kinds), cross
  checked against the parsed file (sections, texts),
  flattened (quotes), or refused (a rule id holding a
  line break, caught by BP-13a). An unvalidated
  evidence kind carrying a newline fabricated a whole
  block in the red team's hands. The replacement's
  lines are split on any line boundary, so no carriage
  return rides a rendered line. Test,
  tests/test_diff.py.
- BP-16a. The output refusal of BP-16 and of propose
  alike also fires when the output path names an input
  by file identity, a hardlink, not only by resolved
  path. Test, tests/test_diff.py and
  tests/test_proposals.py.

## Round 3. The Rust port (BP-18 to BP-26)

Round 3 ports the tool to Rust. The Python on main at
255d973 is the behavioral specification. The port is a
reproduction, not a redesign, and every place where
exact reproduction is impossible is a named deviation
in the round dossier, never a silent approximation.

- BP-18. The Rust crate reproduces the propose and
  diff commands byte for byte across stdout, stderr,
  exit codes, and written files, including refusal
  order, argparse behavior, error text, and the crash
  class where the Python dies without a named refusal.
  Test, the three ported test files and the round's
  differential harness.
- BP-19. The crate is a library with a thin binary.
  Its only external dependency is sha2, for the target
  digest BP-8 names. No sibling crate is consumed,
  because bluepencil reads no tape, record, or other
  sibling format; the JSON readers stand in this crate.
  No Python bindings ship, because the surface is a
  process boundary CLI and its consumers use files and
  stdout. Cargo tests are the primary suite.
- BP-20. The rules parser follows CPython text
  semantics exactly: the wide splitlines boundary set,
  the marker shapes with CPython whitespace and decimal
  digit sets, counts and spans in code points, and the
  parser's decision order across fences, HTML comments,
  setext headings, ATX headings, empty markers, rules,
  continuations, and prose. The unclosed fence warning
  is behavior. Test, tests/test_inventory.rs.
- BP-21. Both JSON input surfaces use one strict
  iterative reader over decoded text. Decisions happen
  in text order. At the shipped CLI a container that
  would open past 9994 nested containers refuses in
  the surface's own way. Duplicate keys follow Python
  dictionary assignment, the last value wins and the
  first key position is retained. Integer literals
  keep their exact digits, an integer literal past
  4300 digits refuses in the surface's own way, and
  NaN and the infinities read as floats.
  Test, tests/test_proposals.rs and
  tests/test_diff.rs.
- BP-22. The observations loader reads its file as
  UTF-8, with no BOM stripping at that surface. A
  failure to read, decode, or parse is reported with
  the CPython exception text the Python embeds,
  including JSON syntax positions in line, column, and
  character terms, the integer digit limit text, and
  the recursion wording for the container kind that
  crosses the reader limit. Validation then proceeds
  in document order, with unknown kinds named by
  CPython repr. Test, tests/test_inventory.rs and
  tests/test_proposals.rs.
- BP-23. Proposal generation keeps the Python's join
  and generation order exactly: per rule, its delete,
  then its supplied rewrites in document order, then
  its promote; a rule text already served by a rewrite
  is not served again; a rewrite naming no rule in the
  file is refused with the name truncated to sixty
  code points and rendered by CPython repr. The
  proposal document keeps its field order exactly, and
  its target is the SHA-256 of the rules file's raw
  bytes. Test, tests/test_proposals.rs.
- BP-24. The diff loader decodes the proposal file as
  utf-8-sig and refuses every decode failure by name.
  Shape checks run before the target check, the target
  check before the fresh rules parse, and the renderer's
  cross-checks before anything renders, in the Python's
  order. Rendering keeps BP-15 exactly, including the
  line boundary discipline of BP-15a.
  Test, tests/test_diff.rs.
- BP-25. The command line surface is argparse at
  eighty columns, with help and usage pinned byte for
  byte. An empty output value means stdout, as the
  Python's own test of the value does. The output
  guard compares resolved paths and file identity, and
  a path that cannot be resolved counts as naming an
  input. Writes are atomic in the mkstemp shape, a
  sibling temporary with the output name and a dot as
  its prefix, no suffix, and mode 0600, replaced on
  success and unlinked on failure. The document text
  is rendered before any temporary exists.
  Test, tests/test_proposals.rs and
  tests/test_diff.rs.
- BP-26. All fifty-nine Python tests are ported file
  for file to tests/test_inventory.rs,
  tests/test_proposals.rs, and tests/test_diff.rs. A
  differential harness runs the whole command surface
  against the Python oracle, comparing exit codes,
  streams, and files after every step. The round
  dossier carries the port report: port or rewrite
  with reasons, the Rust and Python line share on the
  landing tree, the bindings determination, and a
  complete non-Rust remainder register with a written
  justification per item.
- BP-27. Round 4 brings this resolver to the
  finished-path standard its siblings reached.
  Two shapes are probed at the oracle first, and
  the port takes the outcome the oracle dictates
  on each: (a) the cancelled loop, an output
  spelling whose loop component is cancelled by
  a following ".." (pathlib resolves it cleanly;
  a resolver that judges the loop during its
  walk refuses it falsely); (b) the dangling
  final component, an output symlink whose
  target does not exist, where realpath resolves
  the target spelling against the link's parent
  (retroforge's RF-25a shape). Where the port
  diverges, it is repaired in wallofwrong's
  WW-25a detection shape (the loop judged on the
  finished path's prefixes) or retroforge's
  dangling shape, with harness cells on both the
  refusing and agreeing sides and a control run
  proving each new cell discriminates.
- BP-28. The round's verification is the full
  local gate plus the differential harness at
  zero failures, BP-27 cells included.
  CodeRabbit's advisory pass on Round 3's PR 5
  posted no findings (zero entries on the review
  endpoints), so no disposition is owed from it.
- BP-27a. Amendment, from the Round 4 red team.
  The dangling splice iterates: where the
  substituted target spelling itself ends in a
  dangling link, splice again, up to the
  kernel's 40 link bound; a chain still
  unresolvable at the bound counts as naming an
  input under BP-25. And the guard gains the
  oracle's exists arm: _writes_to_input wraps
  the output's exists() in the same try as its
  resolve(), and pathlib exists() propagates
  errnos outside {ENOENT, ENOTDIR, EBADF,
  ELOOP}, so an output whose stat fails that
  way (an unsearchable directory on its path is
  the probed instance) counts as naming an
  input. Mirror the oracle function's call
  structure from its source, and pin both arms
  with harness cells whose control evidence
  discriminates.
- BP-29. Round 5 discharges the probe filed in
  this line's Round 5 owed file, and adds the
  write-seam family the sibling line proved
  live. The deleted working directory: from a
  deleted cwd, a relative output spelling makes
  pathlib's resolve raise FileNotFoundError from
  getcwd; whether this crate's oracle counts it
  as naming an input depends on its guard's
  catch structure (BP-25's try wraps resolve and
  exists together), and the port's cwd fallback
  decides the rest. Relative and absolute
  outputs onto and away, rules and observations
  spellings relative and absolute, on propose
  and diff alike, measured cell by cell against
  the oracle before any repair is written.
- BP-30. The tailed output spelling: retroforge's
  Round 5 red team proved a write-seam class in
  that crate (spellings ending in a separator or
  dot component write on the oracle and refuse
  on the port, because the port hands the raw
  spelling to rename while pathlib's construction
  drops the tail), and wallofwrong's Round 5
  convicted the same class on three commands.
  This crate's write seam is probed for the class
  in the same pass, on propose and diff alike,
  and the class boundary is measured before any
  repair is written.
- BP-31. Repairs follow the measurements under
  new clauses if a probe set expands; the
  differential harness gains the proven cells
  with per-step reseeding where a write-through
  could contaminate later steps, control runs
  against the pre-repair binary fail exactly the
  new cells, and the standing harness stays
  clean.
- BP-32. README records 5 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner verification.
- BP-33. Amendment, from the Round 5 probes
  (BP-29). From a deleted working directory the
  interpreter's resolve raises FileNotFoundError
  from getcwd for any relative spelling, the
  output's or an input's, and this oracle's
  guard catch counts the raise as naming an
  input: every cell with a named output and any
  relative spelling refuses at the guard, exit
  1, nothing written, while all-absolute cells
  behave as in a live directory and cells with
  no output named never reach the guard and
  stand as they are (dot-dot spellings still
  read through the deleted directory's parent
  on both sides). The port's resolver today
  substitutes the root for a failed cwd read
  and manufactures resolutions the guard
  accepts, up to writing the document at exit
  0 where the oracle refuses. Repair: a
  relative spelling whose cwd read fails fails
  the resolution outright, which the guard
  already counts as naming an input; an
  absolute spelling never consults the cwd.
  The iterated-splice resolver is otherwise
  untouched; the probes do not convict it.
- BP-34. Amendment, from the Round 5 probes
  (BP-30). The oracle builds one Path from the
  output argument, and that constructed object
  flows through the guard and the write alike.
  Construction drops every empty component and
  every dot component wherever it stands;
  dot-dot and all other components stand whole;
  a leading separator pair stands as the
  anchor, a longer leading run collapses to
  one; a spelling with no component left stands
  as the dot or as its anchor. The port today
  hands the raw spelling to the guard and the
  write, so a tailed spelling over a link
  escapes the guard's splice and identity arms
  and a tailed spelling anywhere refuses at
  the rename where the interpreter writes.
  Repair: the output spelling is constructed
  once, by the same normalization the port
  already renders path names with, and the
  constructed spelling is what the guard
  resolves and stats and what the write
  replaces; the cannot-write message already
  names the constructed form.
- BP-35. The anchor of a componentless
  spelling: the normalization keeps a leading
  separator pair as the anchor when components
  follow but collapses it when none do, so a
  spelling of separators and dots alone whose
  leading run is exactly two is rendered as
  one separator where the interpreter renders
  the pair (the out "//" cell: both sides
  refuse the write, and the refusal names "/"
  on the port against "//" on the oracle).
  The componentless branch of the construction
  keeps the pair, as the anchored branch
  already does.
- BP-36. The round's verification is BP-28's
  discipline over the new cells: the
  differential harness gains a deleted-cwd
  launch (the command started in a subdirectory
  deleted before it runs, the sandbox reseeded
  before every such step) carrying the BP-33
  convicted and agreeing cells, and tailed
  output cells carrying the BP-34 convictions
  and the agreeing boundaries, on propose and
  diff alike; the control run against the
  pre-repair binary fails exactly the convicted
  cells and passes the agreeing ones.
