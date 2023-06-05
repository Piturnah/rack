# Integration Testing

## Usage

These tests will be run as normal through cargo.

```console
$ cargo t
```

To update the expected output, use:

```console
$ cargo t write -- --write
```

To update the expected output for a specific test case, use:

```console
$ cargo t write -- --write <FILE>
```

> NOTE: Do not include the file extention or the full path. Just the stem.
