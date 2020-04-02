# procon-assistant

Assistant program for programming contest. Supported languages are C++ and Rust.

## How to Setup

### Prerequirements

#### Common

1. your favorite editor

    Any kind of editor works well with this program, but especially Vim or
    Visual Studio Code is useful. Auto-completion and syntax checker integration
    settings are present, since I usually use them to code.

#### C++

1. clang

    clang is used when compiling C++ project.

    **Note for Windows** LLVM for Windows (official build) found at
    <http://releases.llvm.org/download.html> is recommended (MinGW on MSYS2
    version may work, but not tested). Make sure your `clang++.exe` executable
    is placed at the directory which is added to `PATH` environment variable.

#### Rust

1. rustup (or appropriate version of Rust)

    rustup is used when compiling Rust project. If you use rustup you can
    automatically install appropriate version of Rust toolchain.

    **Note for Windows** Visual C++ Build Tools are required to install Rust.
    It's bundled with Visual Studio's C++ installation, so if you have Visual
    Studio installed with C++ functionality on your computer, there's nothing
    else you should do. On the other hand, if Visual Studio is not installed on
    your computer, you'll need to install Visual C++ Build Tools. However, it's
    enough to install just the build tools and doesn't require a full Visual
    Studio installation.

#### Windows

Nothing special is needed.

#### Linux

##### Build Dependencies

* OpenSSL (development)
* pkgconf

if one of them is lacking, simply it doesn't compile.

##### Runtime Dependencies

* xclip (for clipboard functionality)

You can use other features without xclip (though it prints no-such-file error
when trying to use xclip).

### Configuration

If you want to customize the behavior, you can create a config file. The config
file should be placed at `<CONFIG_DIR>/procon-assistant/config.json`, where
`<CONFIG_DIR>` is `%APPDATA%` on Windows, `~/Library/Preferences` on macOS,
`$XDG_CONFIG_HOME` or `~/.config` on Linux.

An example config is:

```json
{
  "general": {
    "editor_command": ["code.exe", "%PATHS%"],
    "wait_for_editor_finish": false
  },
  "init": {
    "auto_open": true,
    "open_target": "directory",
    "default_lang": "cpp"
  },
  "addcase": {
    "give_argument_once": true,
    "editor_command": ["code.exe", "%PATHS%"],
    "wait_for_editor_finish": false
  },
  "run": {
    "timeout_milliseconds": 3000,
    "eps_for_float": 1e-8
  },
  "languages": {
    "rust": {
      "project_template": {
        "type": "git",
        "repository": "https://github.com/rust-lang-ja/atcoder-rust-base",
        "branch": "ja"
      },
      "needs_pre_compile": true
    }
  }
}

```

TODO: descriptions

### Template directory

`<CONFIG_DIR>/template` is a template directory. `<CONFIG_DIR>/template/<LANG>`
is a template directory for each language (`<LANG>` is language id like `cpp`,
`rust`).  The usage of each template directory is completely depends on the
language.

#### `cpp`, C++

In C++, files under the template directory is generated to the project directory
when `init` command is executed. Some variables such as `$PROJECT_PATH` in the
file are replaced to the appropriate values. For example template, this
repository contains `template` directory in the root.

#### `rust`, Rust

Rust does not use the template directory. You can freely use the directory to
place Rust-related resources. For example, you can create a project template
under `<CONFIG_DIR>/template/rust` and specify the directory as a template
directory in the `config.json`.

## Usage

### `initdirs {contest-name} {numof-problems} [beginning-char]`

alias: `id`

creates contest directory tree like following:

```
+- contest-name
   +- a   --+
   +- b     |
   +- c     +-- numof-problems (for example, 5)
   +- d     |
   +- e   --+
```

problem-beginning-char is `a` by default. you can specify it manualy (to use
with AtCoder Regular Contest that begins from Problem C. if you want to make
directory with uppercase, you can specify like `A`) .

### `init {project-dir} [--type,-t lang]`

alias: `i`

creates `main.cpp` in the **project-dir** with a simple template for specified
language (if not, default is for C++) and automatically open it with your
editor. it also creates `.clang_complete` file which contians the include path
for your own library (default: `~/procon-lib`).

### `addcase`

alias: `a`, `ac`

creates new sample case file `inX.txt` and `outX.txt`. `X` is replaced by
not-existing-but-smallest-more-than-zero-integer. They are automatically
opened. paste there sample input and expected sample output.

### `preprocess`

alias: `pp`, `si` (for compatibility; previously this was `solve-include`
command)

preprocesses current solution (solve includes) so that it can be submitted to
the contest. You can pipe them into clipboard using clipboard utilities like
`cliputil` or `xsel`. Note that it doesn't minify the solution, you can use this
to avoid minifying bug (for example, contiguous spaces even in string literal
are reduced by minifier).

### `clip`

alias: `c`

copies the whole contents of `main.cpp` into clipboard. it expands double
quote version of #include directive (`#include "..."`) with your library at
`~/procon-lib`. you don't have to copy-and-paste your library into your code
before submission anymore.

### `fetch {contest-site}:{problem-id}`

alias: `f`

(experimental) fetches sample cases from the contest site. currently only
supports Aizu Online Judge / AtCoder. You can use it like this:

```
% procon-assistant fetch aoj:0000 # Aizu Online Judge Problem id 0000
```

or

```
% procon-assistant fetch atcoder:agc022a # AtCoder Grand Contest 022 problem A
% procon-assistant fetch at:agc022a      # same, you can use `at` instead of `atcoder`
```

for AtCoder, there is also support for problems on non-regular contests: use
the problem's URL directly instead of {problem-id}.

### `download {contest-site}:{contest-id}`

alias: `d`, `dl`

(experimental) downloads the contest webpage, parses its html and determine
the number of problems, initializes directory tree (`initdirs`) and fetch
sample cases for each problem.

```
% procon-assistant download atcoder:agc022 # AtCder Grand Contest 022
```

### `run [sample case using for test]`

alias: `r`

runs test. First it compiles `main.cpp`. If it was not successful, prints
compiler error message and exit. Otherwise, checks that program's output
is the same with `outX.txt` when given `inX.txt` as the standard input.
the test case is existing all cases by default but you can specify using
cases as a parameter. for example, `procon-assistant run 1 3 5` runs
test case 1, 3, 5. Of course sample case files needs to exist.

your program's output is judged and result will be as follows.

- Sample Case Passed

    Your program prints the correct answer at least for the sample inputs.

- Wrong Answer

    Your program prints somehow wrong answer. Input, actual output and
    expected output will be printed. Which line differs will also be
    printed. Check and collect your program and retry.

- Time Limit Exceeded

    Your program exceeds the time limit. Time limit is hard-coded
    (1000ms). Current time limit is displayed when execute `run` command.

- Runtime Error

    Your program caused runtime error. It usually occurs when your program
    returns other than 0. You might cause so called segmentation fault or
    assertion failure. Check and debug your program.
    Note: in Windows, when main.exe caused runtime error, Windows Error
    Report shows "main.exe has stopped working" dialog. While this message
    is shown the program is not stopped. So sometimes Runtime Error may
    treated as Time Limit Exceeded. To prevent this, you can edit registry
    to disable Windows Error Report.

- Presentation Error

    Your program's output is all collect when checks line by line, but
    is different when checks whole output. Maybe you forget newline after
    the last output line.

### `compile`

alias: `co`

compiles your solution. It won't do any test. If you have a lot of error
messages, maybe using `compile` is a better choice, since you can focus on
compiler error messages.

### `login {contest-site}`

alias: `l`

(experimental) logs in to the contest-site. currently only AtCoder is
supported. you can use it for joining to running contest. its problem is only
visible for participants, so you need to log in. note that for now it can't
detect logging in failure (always show as if logging in was successful).

### Note for auto open feature

Note: `init` and `addcase` function has auto-open feature, but this feature
doesn't work unless you don't have `open` (in windows `open.exe`) in your
path.  `open` is an utility taking one argument and open the specified file.
If you don't have `open`, you can implement it by yourself (in Linux, all you
have to do is to write simple wrapper for `gio open` or `xdg-open` or
something. Even symlink to `gvim` may work) or on Windows you can use my
implementation [here](https://github.com/statiolake/open-windows).
