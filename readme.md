
`fcmp` File compare library and command line utility
====================================================

```
Takes a list of file names and returns the most recently modified file.

By default, the file name is returned, and missing files are ignored.

USAGE:
    fcmp [OPTIONS] [PATHS]...

ARGS:
    <PATHS>...
            File paths to compare

OPTIONS:
    -d, --diff
            Ignore files that have the same content

    -h, --help
            Print help information

    -i, --index
            Return the (0-based) index of the file instead of the path

    -m, --missing <MISSING>
            Behavior when comparing missing files

            [default: ignore]
            [possible values: ignore, error]

    -r, --reverse
            Return the oldest file instead of the newest

    -V, --version
            Print version information
```
