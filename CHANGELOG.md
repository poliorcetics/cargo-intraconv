# Version 1.X.Y - ????-??-??

- Added more favored links pattern for `docs.rs`: `https://docs.rs/crate-1`
  will now be transformed to `crate_1` when used in a link.
- Don't remove links that are local but use a disambiguator:
  `/// [tracing]: mod@tracing` was previously removed but the `mod@` part could
  be the only thing helping rustdoc find the correct link and so it is
  necessary to keep the link.

# Version 1.1.0 - 2020-10-29

Git tag: `v1.1.0`.

- Added favored links: by default, `https` link from `docs.rs` will be
  transformed to their intra-doc links version. `intraconv` will assume such
  links point to dependencies and as such can be safely transformed to
  intra-doc links. If this is not the desired behaviour, see the `-f` flag.
- Added `--no-favored` (`-f`) to disable the behaviour described above.

# Version 1.0.1

Git tag: `v1.0.1`.

- Added `--quiet` (`-q`) to suppress changes output.
- Added `--version` to display crate version.
- Filenames are now only displayed when there are changes for the file.
- Filenames are now only displayed once the file has been read and the changes
  found. If an error occurs when reading or finding changes, the filename
  underlined by `=`s will not be displayed.

# Version 1.0.0

Git tag: `v1.0.0`.

First published version and start of the changelog.
