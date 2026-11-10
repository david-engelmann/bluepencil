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
  diverges, it is repaired in tetlock's
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
  drops the tail), and tetlock's Round 5
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
- BP-37. Round 6 probes the temp placement law
  at this crate's write seam, the family
  measured in secondbill's Round 5,
  retroforge's Round 6, and tetlock's
  Round 6, and which this crate's seam
  predates. The oracle's write creates its
  temp through tempfile.mkstemp on the one
  constructed Path the preamble hands to both
  guard and write (BP-34), and mkstemp
  absolutizes the parent lexically (cwd join,
  then normpath, no link resolution); this
  crate's write creates its temp under the
  constructed parent as spelled. The probe
  set is the popped-link family at both
  commands that write (propose and diff): an
  output whose parent contains a symlink
  component that a following dotdot pops,
  across the existence boundary (the lexical
  parent exists as a directory, neither
  parent exists, the popped component is a
  real directory), in plain, tailed,
  absolute, deep, and escape-link spellings,
  each measured against the oracle before any
  repair is written. This crate's guard
  counts an unresolvable output as naming an
  input (BP-25), so the probes also record
  whether the guard fires before the write
  seam is reached on each shape, and a shape
  the guard refuses on both sides is an
  agreement, not a write-seam cell. Repairs
  follow the measurements under new clauses;
  the harness gains the proven cells with
  controls against the pre-repair binary
  failing exactly the convicted cells,
  boundary pins ordered before convicted
  cells where a one-sided write could echo
  through later snapshots.
- BP-38. The CodeRabbit disposition owed from
  Round 5 discharges here on evidence: the
  Round 5 pull request carries zero inline
  findings on read-back (checked 2026-11-04 in
  the rotation-wide late sweep), so there is
  nothing to fix and the absence is the
  disposition.
- BP-39. README records 6 of 100 only in the
  closure commit written after this round's
  three reviews, fixes, and owner
  verification.
- BP-40. Amendment, from the Round 6 probes
  (BP-37). The temp placement law, measured at
  this crate's seam. The oracle's mkstemp
  absolutizes the constructed parent lexically
  (its _mkstemp_inner opens with dir =
  os.path.abspath(dir): a cwd join, then
  normpath, no link resolution anywhere in
  that step), while the port creates its temp
  under the constructed parent as spelled, so
  the kernel resolves a symlink component
  physically before a following dotdot pops
  it. Where the lexical parent and the
  physical parent differ in existence, the
  split runs one way only: the lexical parent
  missing, or standing as a file, while the
  physical parent exists, the oracle refuses
  cannot-write and writes nothing, and the
  port writes the document at the physical
  target at exit 0. Twelve shapes stand
  convicted on propose and diff alike: the
  plain pop, the lexical parent standing as a
  file, an overwrite of a file standing at
  the physical target (the oracle leaves it
  standing, the port replaces it), the tailed
  slash and dot forms, the dotted parent, the
  absolute and pair-anchored absolute forms,
  the deep pop, the escape-link pop (the port
  writes outside the tree the spelling
  suggests), two links popped in sequence,
  and a link whose target itself carries a
  dotdot, in the geometry where its resolved
  self sits under a further directory. The
  guard eats none of the family: a popped
  spelling resolves cleanly for the guard's
  comparison, against resolved inputs that
  live elsewhere, so every convicted shape
  passes the guard on both sides and splits
  only at the temp.
- BP-41. The boundary agreements, also
  measured: where only the lexical parent
  exists, both sides refuse (the oracle at
  its replace, the port at its temp
  creation); where both parents exist, both
  write at the physical target, and a file
  standing at the lexical target stands
  untouched; where neither exists, both
  refuse. A pop of a real directory never
  splits the parents, and both sides write
  or refuse together. A lexical parent that
  is itself a symlink to a directory splits
  nothing (the temp is created through it on
  both sides). A dotdot-carrying target in
  the geometry where its resolved self sits
  at the root makes the parents coincide, and
  both sides write at the lexical target. A
  cancelled loop in the parent (BP-27) passes
  the guard on both sides and refuses
  cannot-write on both at the write; a
  standing loop in the output, and a pop
  landing on an input, refuse at the guard
  on both sides, as before this round.
  Repair, following BP-40: the write creates
  its temp under the lexically absolutized
  parent of the constructed path, by the
  normpath law over the parent's components
  (empty and dot components drop, a dotdot
  pops the spelled component before it, a
  pop at the root drops, a leading separator
  pair stands as the anchor), the parent
  first joined to the working directory when
  it is relative and never consulted when it
  is absolute; a failed cwd read on a
  relative parent fails the write, an arm
  the guard's own BP-33 refusal makes
  unreachable for a named output and which
  stands as the mirror of the interpreter's
  raise inside mkstemp. The rename target
  remains the constructed path itself, the
  cannot-write message keeps the constructed
  spelling, and the guard is unchanged.
- BP-42. The round's verification is BP-28's
  discipline over the new cells: the harness
  gains a popped-link case on propose and
  diff alike, the sandbox wiped and reseeded
  before every step, carrying BP-41's
  boundary pins ordered before BP-40's
  convicted cells, an added abs step form
  (argument substitution of the side's own
  sandbox path, with that path normalized
  out of the compared streams, in plain and
  deleted-cwd launches alike) carrying the
  absolute cells, an in-tree escape form
  whose landing stays inside the snapshot
  (the true escape, landing outside the
  side's root, stands probe-pinned in the
  lane record, as a snapshot cannot carry a
  write outside its tree), and two
  deleted-cwd steps on absolute spellings.
  The control run against the pre-repair
  binary fails exactly the convicted steps
  and passes the pins.

## Round 7

- BP-43. Round 6's CodeRabbit disposition
  stands discharged on read-back, as BP-38
  held for Round 5: the review's record on
  pull request 8, read through the API list
  endpoints on 2026-11-07, carries zero
  review comments and zero reviews; its
  issue-comment thread holds the trigger,
  the summary shell, the landing comment,
  and the review bot's auto-reply. The
  shell's final form is "Review failed"
  over "The pull request is closed", the
  failed-because-closed shape above a
  findings-free walkthrough, and the
  auto-reply reports the action not
  completed because the pull request is
  closed. No late findings exist to
  discharge; the absence is the
  disposition, and no code in this round
  answers to it.
- BP-44. The width arm is convicted at
  both commands. The oracle draws its
  temp suffix from the library default,
  exactly eight characters at every pid
  (a strace cell draws out.json.kae21ppi,
  O_CREAT and O_EXCL, mode 0600), so its
  name-length boundary is fixed: a plain
  output name of 246 bytes writes and 247
  refuses, at the ambient pid, at pid 1,
  and at a three-digit pid alike. The
  port builds its candidate as the name,
  a dot, the decimal pid, a decimal
  counter starting at zero in every
  process, and a decimal attempt, so its
  suffix is the pid's digit count plus
  two at the first draw (a strace cell at
  pid 14488 draws out.json.1448800, seven
  characters, O_CREAT and O_EXCL, mode
  0600). Twin probes measure the flips
  per posture, propose and diff alike:
  at the ambient five-digit pid of the
  probe environment the port writes 247
  and refuses 248; at pid 1 it writes
  251 and refuses 252; at a three-digit
  pid it writes 249 and refuses 250. The
  oracle's flip is 246 and 247 in every
  posture. The shipped suffix matches
  the law only where the pid's digits
  happen to total six; everywhere else
  the port lands writes the oracle
  refuses.
- BP-45. The byte-counting arm is
  convicted at both commands. The port
  builds the candidate's name through a
  lossy conversion, so a name carrying
  one raw byte outside UTF-8 is measured
  as the three-byte replacement
  character, two bytes longer than the
  name on disk. Twin probes with
  one-raw-byte names (a run of a bytes
  closed by byte 0xFF), propose and diff
  alike: at pid 1 the port's flip moves
  from 251 and 252 plain to 249 and 250
  raw; at the ambient pid from 247 and
  248 plain to 245 and 246 raw. The
  oracle counts raw bytes and its raw
  flip stands at 246 and 247, the plain
  flip. The port therefore refuses raw
  names the oracle writes, by exactly
  the lossy expansion, at every pid.
- BP-46. The determinism arm is convicted
  at both commands. The port's draw is a
  public function of the pid, the
  counter, and the attempt, with no
  other input, so the whole drawable set
  at a fixed pid can be enumerated in
  advance: at pid 1 a single write draws
  among exactly 65 candidates, the name
  and dot followed by "1" and the
  counter and attempt digits running
  together (100, 111, 122, and so on to
  16464). Twin probes plant those
  candidates as files, propose and diff
  alike: planting one candidate is
  absorbed by the redraw on both sides,
  and planting all 65 makes the port
  refuse a write the oracle lands,
  printing its cannot write the proposal
  file (diff, the diff file) refusal for
  a name that carries no other obstacle.
  A planted directory at a known pid can
  therefore deny the port its output
  outright, where the oracle's draw
  cannot be enumerated at all.
- BP-47. The repair rebuilds the
  candidate from raw bytes end to end
  and draws exactly eight characters:
  the output name's own bytes, a dot,
  and the first four bytes of a SHA-256
  digest rendered as eight lowercase hex
  characters, the digest taken over the
  process id, the wall-clock time in
  nanoseconds, the write counter, the
  attempt, the temp directory's bytes,
  and the name's bytes (the FT-74 mix
  shape). This crate reads no wall clock
  anywhere before this repair; the read
  is new here and is owed to BP-46's
  conviction, a clock-free draw having
  stood convicted rather than merely
  filed. The retry cap, the 0600 mode,
  the placement under BP-41's lexical
  absolute parent, the rename target,
  and the cannot-write text stand
  unchanged, and no manifest change is
  needed, the digest crate already being
  a dependency.
- BP-48. The README is brought to the
  fleet standard in this round: title,
  a lede identical to the repository
  description, one short paragraph, the
  ci workflow's badge, one block of real
  captured output, then Install,
  Quickstart, Usage, Status, and License
  sections, the Install and Quickstart
  blocks run verbatim before landing and
  every output block captured from a run
  of the repaired binary. The badge is
  verified against the workflow's live
  state on main, the ci workflow active
  and its latest run at the round's base
  completed in success; the badge's own
  image fetch was not observed in this
  lane and the clause claims the
  workflow-state form of the
  verification, not a render. The
  standard's identity rule carries one
  recorded exception: the description,
  and so the lede, holds a single colon,
  which the line keeps verbatim because
  the identity rule is the stronger rule
  (the OR-39 precedent); no other colon
  or semicolon stands in the README's
  prose. Item 3 of the standard's
  checklist is recorded open: the crate
  manifest carries no description field
  at all, so the metadata side of the
  identity is absent rather than a
  differing text; it is named here and
  not repaired in this round. The change
  is the README only, and the Status
  line keeps the closed-rounds count.
- BP-49. The round's verification is
  BP-28's discipline over the new cells:
  the harness gains two pid-posture step
  forms (a pid-namespace launch, and the
  same launch behind a launcher that
  burns 150 pids first so the tool runs
  at a three-digit pid) and a temp suffix
  boundary case on propose and diff
  alike, the sandbox wiped and reseeded
  before every step, carrying the plain
  boundary at the ambient pid, at pid 1,
  and at a three-digit pid, the
  one-raw-byte boundary at pid 1 and at
  the ambient pid, and the plant cells
  at pid 1, each convicted arm pinned at
  its own flip point rather than at the
  shipped combination's alone (the WW-56
  battery-design rule). The control run
  against the pre-round binary fails
  exactly the convicted cells, and two
  half-repair mutants, one keeping the
  lossy name over a repaired draw and
  one keeping the shipped generator over
  a repaired name, are each convicted at
  their own arm's cells and pass the
  other's.
- BP-50. The closure moves the README
  rounds line to 7 of 100 and is the
  only later change to that line this
  round makes.

- BP-47a. One posture inside BP-47's repair
  is named with it: where the wall clock
  cannot be read, or reads before the
  epoch, the write refuses rather than
  drawing on a term an observer could
  supply. A fixed fallback would hand the
  enumeration of BP-46 back to any pid
  that could arrange the clock's failure,
  so the refusal is the repair's shape,
  at the price of refusing in a posture
  the oracle's library draw survives.

- BP-51. Two review dispositions and one
  mechanism distinction are recorded at
  closure. The first disposition is the
  standards lane's copy-bar Minor: the
  README's Status prose carried the
  clause range "(BP-1 through BP-50)",
  and decision codes are barred from
  README prose, so the range is dropped
  in this closure and the sentence keeps
  the CONTRACT.md reference in plain
  words. The second is BP-49's evidence
  state. The lane's own mutant logs were
  truncated to zero bytes by the box
  write fault before they could be read,
  so the half-repair convictions BP-49
  states rest on the red team's
  rebuilds: both mutants rebuilt from
  fresh archives of the tip, each in an
  isolated target dir, the md5
  divergence check passing over the
  pre-round binary, the repaired binary,
  and both mutants, the runs banked in
  the lane's scratch as
  harness-redteam-mutant-w.txt,
  harness-redteam-mutant-l.txt, and
  harness-redteam-repaired.txt. The
  mutant keeping the lossy name over the
  repaired draw fails exactly 4 steps,
  every one a raw-byte cell at the
  counting arm's own flip length; the
  mutant keeping the shipped decimal
  generator over the repaired raw-byte
  name fails exactly 14 steps, the
  plant-all cell at pid 1 and the width
  cells at their own flip points; the
  repaired build reads 15 cases, 574
  steps, 0 failures over the full
  extended harness. BP-49's claims
  therefore stand on banked runs, cited
  by log name here. The mechanism
  distinction, from the red team's clock
  battery, concerns BP-47a. The refusal
  boundary sits exactly at the epoch,
  neither wider nor narrower than the
  clause states: at tv_sec -1 the
  repaired binary refuses at both
  commands through the crate's own Err
  arm on duration_since, exit 1, no
  output file and no temp residue, and
  at tv_sec 0 and tv_sec 1 the write
  lands. Under the harder failure,
  clock_gettime returning EINVAL, the
  write still refuses, exit 1, nothing
  written, but the refusing agent is the
  standard library, not the crate's Err
  arm: SystemTime::now unwraps the
  failed call inside std and panics in
  the stack thread, whose join failure
  surfaces as the ordinary exit 1 with
  the panic line on stderr. The clause's
  observable contract, refuse rather
  than draw on a term an observer could
  supply, holds on both halves of
  "cannot be read, or reads before the
  epoch"; the graceful path covers the
  epoch half and std covers the
  unreadable half. Only the mechanism
  differs from the clause's plain
  reading at that extreme, and no cell
  lands a degraded draw, which is what
  the clause exists to prevent.

- BP-52. CodeRabbit's step 0 pass on the
  Round 7 pull request (pull request 11)
  posted three review comments carrying
  four findings, all against the README
  the round had just written, and all
  four stand repaired in this clause's
  commit. Two findings complete the
  Usage account. The thresholds BP-10
  contracts are now stated where a
  reader meets the proposals, a delete
  proposal needing at least two ignored
  observations and no followed one, and
  a promote proposal needing at least
  two followed observations, the rule
  in the second half of the file by rule
  order, and a section different from
  the first rule's section, each
  condition read off the generation code
  before the sentence was written. The
  rewrite input BP-9 contracts is stated
  with them, a misreading observation
  alone producing no rewrite, a rewrite
  coming only from a supplied rewrites
  entry naming the rule by exact text,
  quoting a misreading observation
  recorded against that rule, and giving
  the replacement, with an entry whose
  quote matches nothing on its rule
  refusing the run by name. The third
  finding is the captured documents.
  Both JSON blocks in the new README
  were prefixes cut mid-object, neither
  valid JSON and neither showing a
  proposal. The first screen now prints
  a complete proposal document, the
  Quickstart run's own output at 1,702
  bytes, proposal and evidence whole,
  captured from the installed binary
  over the Quickstart inputs as written
  on the page. The Usage section drops
  its excerpt of the synthetic document,
  which stands at 4,991 bytes, and
  describes the document's contents in
  prose instead, so no excerpt stands
  anywhere as a capture. The fourth
  finding is the Install block, which
  opened at the install command with no
  way to reach the crate. It now opens
  with the clone and the change of
  directory, and the Install and
  Quickstart blocks re-ran verbatim from
  clean staging into a scratch cargo
  home, the install landing the binary
  and the Quickstart outputs
  byte-identical to the page.

## Round 8

- BP-53. The round opens with the full
  read-back of pull request 11, every
  review body to its end, every review
  comment thread, and every issue
  comment, the follow-up shells and the
  acknowledgment replies included. Four
  of CodeRabbit's findings stand
  disposed in BP-52 and on the pull
  request, two of their threads
  confirmed addressed by the bot
  itself. Two items stood undisposed.
  The first is this round's second
  charge: the review body's additional
  comment, a Minor on security, marked
  unverified by its author, holding
  that SHA-256 over public inputs does
  not make the temp-name draw
  unpredictable, a writer who knows a
  fixed clock value and the next pid
  and counter being able to calculate
  and plant every candidate for the 65
  attempts, so BP-47's draw does not
  close BP-46's planted-candidate
  failure under that condition, and
  asking for operating system
  randomness in each draw with refusal
  when that source fails. BP-54
  adjudicates it by measurement and
  BP-55 repairs it. The second is the
  acknowledgment reply on the
  thresholds thread, posted after the
  owner's replies on the final head,
  holding that two details remained in
  the README and asking that the
  rewrite entry's fields be named; all
  three points are repaired in this
  item's commit, and their ground is
  stated here exactly. The proposals
  paragraph said that below either
  threshold a rule earns no proposal,
  which a supplied rewrites entry
  answers with a rewrite proposal
  below both thresholds; the sentence
  now claims no delete or promote
  proposal below either threshold.
  The introduction said rules agents
  misread are proposed rewritten,
  where a misreading alone produces
  no rewrite; it now says supplied
  rewrites of misread rules are
  proposed. And the rewrite paragraph
  now names the entry's fields, rule,
  misreading_quote, and replacement,
  where it had described them without
  naming them. The remainder of the
  record owes nothing: the summary
  shell is the rate-limit banner over
  a complete walkthrough of the final
  head, its Docstring Coverage check
  passed at 100.00 percent over one
  function, the follow-up command
  shell is the plain rate-limit form
  with no findings, and the remaining
  reviews are empty-bodied reply
  parents.

- BP-54. The frozen-clock adjudication
  convicts BP-47's draw, confirming
  the unverified finding of BP-53 by
  measurement. The battery freezes
  the wall clock by interposition at
  two readings, 1,700,000,000.123456789
  and 1,800,000,000.987654321, fixes
  the pid in a pid namespace at 1 and
  at 250, and computes the draw's
  candidates in advance over the
  contracted mix, the counter and the
  attempt running in lockstep from
  zero in a fresh process's single
  write. At the first reading and pid
  1, planting all 65 candidates
  refuses the write, exit 1 with the
  cannot write the proposal file
  refusal and the directory left
  exactly as seeded, and planting 64
  writes on the sixty-fifth candidate,
  the document byte-identical to the
  unshimmed baseline. At the second
  reading and pid 250 the same split
  stands, 65 refusing and 64 writing.
  SHA-256 over the pid, the frozen
  clock, the counter, the attempt,
  the directory's bytes, and the
  name's bytes is therefore a public
  function of inputs an observer can
  know or fix, and a planted
  directory can deny the write
  outright under a fixed clock, the
  failure BP-46 convicted in the draw
  the hash replaced.

- BP-55. The repair adds the entropy
  term. The mix keeps BP-47's order,
  process id, nanoseconds, counter,
  attempt, the directory's bytes, and
  the name's bytes, and appends
  sixteen bytes of operating system
  randomness, drawn fresh per
  candidate from /dev/urandom through
  the standard library's file read.
  Any failure of that source, the
  open or a short or failed read,
  refuses the write; the draw never
  falls back to the computable mix,
  and the non-unix arm, which has no
  such source to read, refuses every
  write rather than drawing
  computably. The draw factors into a
  terms function the tests pin term
  by term and a live draw that yields
  None on an unreadable or pre-epoch
  clock (BP-47a, unchanged) or an
  unreadable entropy source. The pins
  are the entropy term deciding the
  suffix with every other term fixed,
  the suffix matching an independent
  derivation from the contracted
  layout, and a live draw rendering
  eight lowercase hex characters or
  refusing, two draws not being
  required to differ, the suffix
  carrying thirty-two bits of the
  hash, so no collision-freedom is
  claimed for it. The battery of
  BP-54 rerun against the repaired
  binary at the first reading and pid
  1 leaves all 65 clock-only plants
  inert, the write landing with the
  document byte-identical to the
  baseline, and under an interposition
  that fails every open of
  /dev/urandom with EACCES the write
  refuses with the directory left
  exactly as seeded; the pre-epoch
  refusal also stands measured again.
  The retry cap, the 0600 mode, the
  placement, the rename target, and
  the cannot-write text stand
  unchanged, and no manifest change
  is needed, the digest crate already
  being a dependency and the entropy
  read being the standard library's.

- BP-56. The observations document's
  version key is consumed, completing
  the joint repair unheard's UH-59
  opened: the document is versioned
  on the writing side there and is
  read and checked on this side here,
  and the acceptance law stands in
  this contract in its own text. The
  shipped loader's behavior is
  measured first: it read no version
  key at all, accepting a document
  carrying observations_version
  "1.0", a document carrying "2.0",
  the number 2, a list holding "1.0",
  and the empty string exactly as it
  accepts a document with no key, the
  key wholly invisible to it. The
  loader now checks the key when
  present, after the document's shape
  gate, whose refusal keeps its
  precedence: the string "1.0", the
  version UH-59 defines, is accepted,
  and any other value, of whatever
  type, refuses the load by name as
  unsupported observations_version,
  with the value rendered as the
  loader renders an unknown kind,
  since a version this reader does
  not know may name a shape it cannot
  absorb. A document with no version
  key loads as it always has: absence
  can only be a pre-version document
  or a hand-written one, both of the
  shape BP-6 defines, and the
  Quickstart's own document carries
  no key. BP-6's definition gains the
  key by this item's statement: the
  observations document may open with
  observations_version, and a reader
  of this contract knows its law
  without opening unheard's. The
  battery's version cells measure
  the split on both binaries, and
  the loader's pins cover the
  accepted key, the refused values,
  and the versioned document loading
  as its unversioned twin.

- BP-57. The round's verification is
  BP-28's discipline over the new
  cells. The differential harness
  gains the observations version
  case: the "1.0" arm compared
  against the oracle, which reads no
  version key and so accepts what
  the port accepts, outputs and
  trees byte for byte, and two solo
  arms, "2.0" and the number 2,
  pinning each side's own outcome,
  the port's named refusal with
  nothing written against the
  oracle's write, the divergence
  itself being the contracted
  behavior of BP-56. The harness
  stands at 16 cases, 577 steps, 0
  failures against the repaired
  binary, and the control run
  against the pre-round binary fails
  exactly the new case's two solo
  arms and nothing else. The battery
  of BP-54 and BP-55 stands in the
  round's scratch as the draw's
  instrument, the frozen-clock cells
  being outside the harness's step
  forms, as Round 7's clock battery
  stood outside it (BP-51). Cargo
  stands at 64 tests, the 59 of
  Round 7 plus the draw's three pins
  and the loader's two, with clippy
  under -D warnings and fmt both
  clean on the final tree.

- BP-58. The closure moves the README
  rounds line to 8 of 100 and is the
  only change to that line this round
  makes, the README's three repaired
  sentences having landed with BP-53.

- BP-59. CodeRabbit's step 0 pass on
  the Round 8 pull request (pull
  request 12) posted one review
  comment and one finding in its
  review body, both Minor, both
  against the README, and both stand
  repaired in this item's commit. The
  comment holds that BP-53's repaired
  threshold sentence still overclaims
  in its tail: a rule with a single
  ignored observation is not left
  alone, since two followed
  observations with the position and
  section conditions earn it a
  promote proposal and a supplied
  rewrites entry earns it a rewrite.
  The tail now claims only that a
  single ignored observation by
  itself earns a rule no delete
  proposal, which is the threshold
  the sentence illustrates. The body
  finding holds that the non-unix
  refusal of BP-55 is nowhere stated
  where a user meets it: the
  Quickstart now states, after its
  run, that writing to a file draws
  its temporary name with sixteen
  bytes from the operating system's
  randomness device at /dev/urandom,
  that a build on a platform without
  that device refuses the write
  rather than drawing a computable
  name, and that stdout, which needs
  no draw, works on every build.

- BP-60. The same pass's summary
  shell carries one further item,
  recorded here so the record stands
  whole: its Docstring Coverage
  check reports 44.44 percent
  against a threshold of 80.00
  percent, over nine functions
  analyzed across three files, and
  the shell is the rate-limit banner
  over a complete walkthrough of the
  head it reviewed. The item is
  dismissed. It names no function,
  this repository carries no
  docstring configuration or
  threshold of its own for the check
  to enforce, and the check applies
  the review tool's default profile.
  The basis is the one secondtake's
  ST-61 and unheard's UH-57 state
  for the same item on their own
  pull requests.
- BP-61. Two record items close
  Round 8. First, BP-57's harness
  sentence is qualified. The run
  it reports, 16 cases, 577
  steps, 0 failures, ran on the
  binary built from the gated
  slice tree. No code changed
  after the slice. The closure,
  the BP-59 repair, and the
  BP-60 disposition touch only
  CONTRACT.md and README.md, and
  the standards repair that
  followed the reviews touches
  only a doc comment in
  src/cli.rs, proven comment-only
  by a byte-identical stripped
  remainder. The battery's 22
  cells ran on the final binary,
  and cargo test 64 and hosted
  CI stand on the final head,
  so the run BP-57 reports is
  evidence for the code that
  shipped. Second, CodeRabbit's
  follow-up review 5820358596
  on the repair head carries one
  item, the Docstring Coverage
  recurrence, and it is dismissed
  on the basis BP-60 states. The
  standards repair itself stands
  recorded here. The write's doc
  paragraphs sit above
  write_atomically again, and the
  pull request body names the
  final head 27deba11 and the
  range BP-53 to BP-60.

## Round 9

- BP-62. The round opens with the full
  read-back of pull request 12, every
  review body to its end, every review
  comment thread, and every issue
  comment, the follow-up shells and the
  acknowledgment replies included. The
  pass carries four reviews, three
  review comments, and five issue
  comments. Its two findings, the
  threshold sentence's tail and the
  non-unix output limitation, both
  Minor and both against the README,
  stand repaired in BP-59, the comment
  thread confirmed addressed by the
  bot itself. The summary shell's one
  check item, Docstring Coverage,
  stands dismissed in BP-60, and the
  follow-up review 5820358596 on the
  repair head, which carries that
  item's recurrence and nothing else,
  stands dismissed in BP-61 on
  BP-60's basis. The shell's final
  form reports the check at 55.56
  percent where the form BP-60 read
  reported 44.44; the item is the
  same in both forms, a coverage
  figure over nine functions across
  three files naming no function, so
  the dismissal covers the final form
  as it covered the first, and the
  shell's merge-risk sentence restates
  the non-unix item BP-59 repaired,
  the Quickstart sentence it asked
  for standing in the README. No item
  from the pass stands undisposed, so
  this round's read-back charge
  closes with no repair owed.

- BP-63. The observations version law
  of BP-56 is measured fresh, at the
  shipped binary against the oracle,
  over twenty cells. A document with
  no version key and a document
  carrying the string "1.0" load on
  both sides, to stdout and to a
  file, the output byte-identical to
  the unversioned baseline. Eight
  further values refuse the load on
  this side by name, each rendered
  as the loader renders an unknown
  kind: the string "2.0", the empty
  string, the integer 2, the float
  1.0, true, null, a list holding
  "1.0", and an object. The oracle,
  which reads no version key, accepts
  and writes every one of them; the
  split is the contracted behavior of
  BP-56, not a drift. A document
  carrying the key twice takes the
  last value on both readers, an
  accepted "1.0" standing last
  loading and a refused "2.0"
  standing last refusing. The shape
  gate keeps its precedence: a
  document whose observations entry
  is not a list meets the shape
  refusal on both sides, its version
  unexamined. The version check in
  turn precedes entry validation: a
  bad entry under a refused version
  meets the version refusal here and
  the entry refusal on the oracle,
  and under "1.0" both sides name
  the entry. A refused load writes
  nothing. The law's statement is
  BP-56's and is unchanged; this
  item is its fresh evidence.

- BP-64. The temp-name draw's posture
  of BP-55 is measured fresh over
  eight cells. The clock and entropy
  interpositions are Round 8's,
  carried into this round's scratch;
  the cells are this round's
  battery's own. The 65
  candidates computed over the
  pre-repair mix, planted at
  1,700,000,000.123456789 with pid 1
  and at 1,800,000,000.987654321 with
  pid 250, are inert on the shipped
  binary: the write lands at both
  readings and the document is
  byte-identical to the unshimmed
  baseline. An entropy source whose
  open fails refuses the write on
  propose and on diff, the directory
  left exactly as seeded, and a
  source that opens but reads short,
  the randomness device redirected
  to /dev/null so the read comes up
  empty, refuses the same way; that
  arm is new this round. A clock
  reading one nanosecond before the
  epoch refuses both commands,
  BP-47a standing. The oracle writes
  under both interpositions, its own
  temp naming never opening the
  randomness device; the difference
  is one of mechanism, measured and
  recorded, and no contracted
  behavior splits on it. The draw's
  per-candidate freshness stands on
  BP-55's factoring and its pins;
  this battery measures the
  single-candidate write and the
  failure arms.

- BP-65. The invariants of BP-4 and
  BP-12 are measured fresh under
  adversarial outputs, over
  thirty-four cells, twenty-three on
  propose and eleven on diff, every
  paired cell byte-identical across
  the port and the oracle in exit
  code, streams, and the file tree
  after the run. The guard refuses
  an output naming an input by the
  direct spelling, by a dot or
  dotdot spelling, by a trailing
  slash, by symlink, by hardlink,
  through a symlinked parent, by a
  dangling splice whose target
  spelling lands on an input, through
  a chain of forty-one links, and as
  a bare loop, each refusal the
  never-writes error with no file
  changed. The same file standing in
  both input seats is read as the
  second seat's document and refused
  by that seat's loader, nothing
  written, on both commands. A valid
  write changes nothing but its
  target: the inputs stand
  byte-identical, the output lands
  at mode 0600, and no temp residue
  remains. An output that is a
  symlink to a safe file is itself
  replaced, its target untouched,
  and a dangling symlink to a fresh
  name is itself replaced, its
  target never created. An output
  that cannot land, a missing
  parent, a parent that is the rules
  file itself, or an existing
  directory, meets the named
  cannot-write error and leaves no
  residue. The cancelled loop splits
  as BP-27 states it: a loop
  component cancelled by a following
  dotdot names no input on the
  finished path, so the out.json
  spelling passes the guard and
  refuses at the write seam, the
  temp's parent being spelled
  through the loop, while the
  rules.md spelling in the same
  shape refuses at the guard. No
  apply command exists: the name
  meets the usage refusal at exit 2
  with identical streams on both
  sides, and a rendered diff changes
  no file it renders from. The
  unsearchable-directory stat arm of
  BP-27a is not re-probed here; it
  stands on its Round 4 measurement,
  this battery's shapes being the
  ones named above.

- BP-66. The round's verification is
  BP-28's discipline over the
  re-measured surfaces. This round's
  battery stands at 62 cells, 0
  failures, its version, draw, and
  guard cells the instruments of
  BP-63, BP-64, and BP-65. The
  differential harness stands at 16
  cases, 577 steps, 0
  failures against the shipped
  binary. The round changes no code:
  cargo stands at 64 tests, clippy
  under -D warnings and fmt both
  clean on the final tree, and the
  codepoint scan stands at zero on
  both dash codepoints over the
  changed files.

- BP-67. The closure moves the README
  rounds line to 9 of 100 and is the
  only change to that file this round
  makes, the round's slice being the
  contract items above.
