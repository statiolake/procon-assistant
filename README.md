# procon-assistant

assistant program for programming contest.

## How to Setup

### Prerequirements

#### Common

1. clang

    clang is used to compile the binary.

    **Note for Windows** LLVM for Windows (official build) found at
    <http://releases.llvm.org/download.html> is recommended (MinGW on MSYS2
    version may work, but not tested). Make sure your `clang++.exe` executable
    is placed at the directory which is added to `PATH` environment variable.

1. your favorite editor

    Any kind of editor works well with this program, but especially Vim or
    Visual Studio Code is useful. Auto-completion and syntax checker
    integration settings are present, since I usually use them to code.

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

### Create config.json

This program needs `config.json` in the same directory with the executable
(usually `target/release/config.json` or somewhere).

An example config is:

```json
{
  "editor": "/path/to/your/favorite/editor",
  "is_terminal_editor": false,
  "init_auto_open": true,
  "init_open_directory_instead_of_specific_file": false,
  "init_default_lang": "cpp",
  "addcase_give_argument_once": false
}
```

Short description for each variable is as follows:

| name | type | description |
|------|------|-------------|
| editor | boolean | The path of your editor |
| is_terminal_editor | boolean | if the editor is used within the terminal; this matters `addcase` command waits the editor's finish or not. `addcase` opens two file, `inX.txt` and `outX.txt`. When true, addcase waits the editor's finish before opening `outX.txt`. Otherwise `addcase` spawns the editor opening `outX.txt` before the finish of the first editor opening `inX.txt`. |
| init_auto_open | boolean | If true, open initialized file or project with the editor to start coding. |
| init_open_directory_instead_of_specific_file | boolean | If true, the project directory name is specified instead of the `main.cpp`. This is useful for Visual Studio Code, which treat one directory as a workspace. |
| init_default_lang | string | The default language to be initialized if the language is not specified by the command-line argument. currently, `cpp` and `rust` are supported. |
| addcase_give_argument_once | boolean | If true, two filenames (`inX.txt`, `outX.txt`) will be passed to the editor at once.  |

### Copy or create symlink (or junction in Windows) to `template` directory

`init` command generates various files. Templates for these generated files
are in `template` directory, so this program needs `template` directory is at
the same place the executable exists. Of course, you can simply copy them to
the same directory, but to avoid inconsistency I recommend making them as a
link.

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
