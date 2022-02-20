
`fcmp` File compare library and command line utility
====================================================

```
Takes a list of file names and returns the most recently modified file.

If the result would be ambiguous, the first occurring ambiguous item in the file list will be
returned.

USAGE:
    fcmp [OPTIONS] [PATHS]...

ARGS:
    <PATHS>...
            File paths to compare

OPTIONS:
    -d, --diff
            Consider files with the same content as equal

    -h, --help
            Print help information

    -i, --index
            Return the (0-based) index of the file instead of the path

    -m, --missing <MISSING>
            Determines how to handle missing files.

            By default, missing files will be treated as older than all other files.

            [default: oldest]
            [possible values: oldest, newest, ignore, error]

    -r, --reverse
            Return the oldest file instead of the newest

    -V, --version
            Print version information
```
