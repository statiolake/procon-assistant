# procon-assistant: assistant program for programming contest.

## Usage

### `initdirs {contest-name} {numof-problems} [problem-beginning-char]`

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

problem-beginning-char is`a` by default. you can specify it manualy (for
example, AtCoder Regular Contest begins from Problem C. if you want to make
directory with uppercase, then you should specify `A`) .

### `init`

alias: `i`

creates `main.cpp` in the **current directory** with a simple template
(hard-coded) and automatically open it.

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
testcase 1, 3, 5. Of course sample case files needs to exist.

your program's output is judged and result will be as follows.

- Accepted

    Your program prints the correct answer. Your program is accepted (as
    far as with sample cases).

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

### Note for auto open feature

Note: `init` and `addcase` function has auto-open feature, but this feature
doesn't work unless you don't have `open` (in windows `open.exe`) in your
path.  `open` is an utility taking one argument and open the specified file.
If you don't have `open`, you can implement it by yourself (in Linux, all you
have to do is to write simple wrapper for `gio open` or `xdg-open` or
something. Even symlink to `gvim` may work) or on Windows you can use my
implementation [here](https://github.com/statiolake/open-windows).
