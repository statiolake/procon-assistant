# procon-assistant: assistant program for programming contest.

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

### `init`

alias: `i`

creates `main.cpp` in the **current directory** with a simple template
(hard-coded) and automatically open it. it also creates `.clang_complete` file
which contians the include path for your own library (hard-coded, default:
`~/procon-lib`).

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

### `addcase`

alias: `a`, `ac`

creates new sample case file `inX.txt` and `outX.txt`. `X` is replaced by
not-existing-but-smallest-more-than-zero-integer. They are automatically
opened. paste there sample input and expected sample output.

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
