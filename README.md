# mdbook-autosummary

Generate a `SUMMARY.md` for your [mdBook](https://github.com/rust-lang/mdBook) based on your folder structure!

> [!WARNING]
> The implementation is hacky and has several limitations, see [below](#limitations) for more info.

## Example

```

```

## Installation

### From source

To install from source, you need a working installation of the [Rust toolchain](https://rustup.rs/)

After installing the toolchain, run:

```sh
cargo install mdbook-autosummary
```

## Usage

To use the preprocessor, add the following to your `book.toml` file:

```toml
[preprocessor.autosummary]

# This is so that mdBook doesn't start regenerating 
# deleted folders before autosummary can remove them from SUMMARY.md
[build]
create-missing = false
```

### Usage with other preprocessors

> [!WARNING]
> This MUST be the first preprocessor to run, otherwise the outputs of other preprocessors may be ignored!

To ensure this, add the following to the other preprocessors in your `book.toml` file:

```toml
[preprocessor.foo]
after = ["autosummary"]
```

It is also recommended to add `SUMMARY.md` to `.gitignore`.

### Requirements

- All folders that you want to be included in the `SUMMARY.md` **must** have an `index.md` (or equivalent) file in them. This is the file that will be linked to in the `SUMMARY.md`.
  - Folders that do not have `index.md` (or equivalent) in them will be ignored.
- All files should begin with an h1 (`# `) heading that will be used as the title of the page in the `SUMMARY.md`.
  - The file/folder's name will be used as fallback if no h1 heading is found.

## Configuration

All configuration options are optional and go under the `preprocessor.autosummary` table in `book.toml`.

```toml
[preprocessor.autosummary]
# Controls the name of the index.md file that is looked for in each folder.
index-name = "index.md"
# If true, files that start with . or _ are ignored.
ignore-hidden = true
```

## Limitations

If a `SUMMARY.md` doesn't exist when `mdbook build` is run or is invalid, mdBook by default fails the build *before* calling any preprocessors.

This has the following implications:

- When deleting files or folders that were in `SUMMARY.md` before, `SUMMARY.md` has to be emptied/truncated manually to force regeneration.
- A `SUMMARY.md` must exist, even if it is empty, before running `mdbook build`.

Therefore it is recommended to not commit `SUMMARY.md` to source control and adding it to `.gitignore`. In addition, in CI/CD, an empty `SUMMARY.md` must be created before running `mdbook build`,
even with this preprocessor active.

## How does it work?

The way it works internally is a *bit* hacky due to the fact that preprocessors are not supposed to modify `SUMMARY.md` directly. 

When the preprocessor is invoked, it generates a new `SUMMARY.md` in memory & compares it to the existing one. If they do not match, the generated one overwrites to existing one, and the book is reloaded from disk entirely. If they do match, the preprocessor does nothing. To my knowledge this is the only way to force `mdBook` to reparse the `SUMMARY.md` file, short of parsing & building a `Book` object yourself from scratch.

The side-effects of this approach are that mdBook is forced to reload the entire book when `SUMMARY.md` changes, and possibly invoke some preprocessors multiple times as well. (If they are defined before `mdbook-autosummary`) Also since this is a preprocessor, it is invoked for every backend that is built, although filehash comparisons are used to avoid unnecessary work.

### Why not just tell mdBook to use the new Summary you generate?

The function to do that [exists](https://github.com/rust-lang/mdBook/blob/master/src/book/book.rs#L212) in `mdBook` but is unfortunately private. Given the hacky nature of this preprocessor, I don't think it's worth the effort to make a PR to `mdBook` to make it public, instead, alternative approaches to this problem should be explored. (For example adding the ability to specify the source of a summary to be an extension/preprocessor rather than a file)
