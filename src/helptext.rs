//! The help and usage texts, pinned byte for byte
//! from the Python's argparse at COLUMNS=80 (the
//! family's standing pin; the Python wraps to the
//! terminal width, the port stays at 80).

pub const TOP_HELP: &str = "\
usage: bluepencil [-h] {propose,diff} ...

positional arguments:
  {propose,diff}
    propose       propose edits to a rules file
    diff          render proposals as a diff for a human

options:
  -h, --help      show this help message and exit
";

pub const TOP_USAGE: &str = "usage: bluepencil [-h] {propose,diff} ...";

pub const PROPOSE_HELP: &str = "\
usage: bluepencil propose [-h] [-o OUTPUT] rules observations

positional arguments:
  rules
  observations

options:
  -h, --help            show this help message and exit
  -o OUTPUT, --output OUTPUT
";

pub const PROPOSE_USAGE: &str = "usage: bluepencil propose [-h] [-o OUTPUT] rules observations";

pub const DIFF_HELP: &str = "\
usage: bluepencil diff [-h] [-o OUTPUT] rules proposals

positional arguments:
  rules
  proposals

options:
  -h, --help            show this help message and exit
  -o OUTPUT, --output OUTPUT
";

pub const DIFF_USAGE: &str = "usage: bluepencil diff [-h] [-o OUTPUT] rules proposals";
