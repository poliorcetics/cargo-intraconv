# Version ?.?.? - ????-??-??

- `index.html#section` now correctly always link to `self`, not the current
  page.

# Version 1.2.0 - 2020-11-19

- Added more favored links pattern for `docs.rs`: `https://docs.rs/crate-1`
  will now be transformed to `crate_1` when used in a link.
- Even more favored links: `https://doc.rust-lang.org` is now supported.
- Don't remove links that are local but use a disambiguator:
  `/// [tracing]: mod@tracing` was previously removed but the `mod@` part could
  be the only thing helping rustdoc find the correct link and so it is
  necessary to keep the link.
- Not giving any path to `cargo intraconv` was an error before. Now it just
  find the current workspace (either one crate or a group of crates) and search
  the links in the `src` directories. This search is recursive. It is still
  possible to give only one file to `cargo intraconv` and it will only
  consume this file.
- Correctly transform `[name]: ../metadata` to `[name]: super::metadata`.
- Correctly transform `[name](../metadata)` to `[name](super::metadata)`.
  This will also correctly delete the `(link)` part if the transformation
  produces something of the form `[name](name)`.
- Type blocks are never added before a lone section. This means a link like
  `[name]: #section` will never change.
- Add an option to ignore some links through a file, either for all visited
  files or for some specific files.

## Internals

The internals of the crate have been extensively rewritten. They now use a 
cleaner link parser that is more safe and more extensible.

This was made necessary by #21, which asked for the support of `[name](link)`.
Doing this with the previous internals would have duplicated pretty much
everything. This is not the case anymore.

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
