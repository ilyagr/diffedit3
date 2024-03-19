This test data is not currently *used* by any automatic tests. You can use it
for manual testing. E.g.:

```
$ cargo run -p diffedit3 -- testdata/{left,right,edit}
```

or if the shell does not support this kind of expansion:

```
$ cargo run -p diffedit3 -- testdata/left testdata/right testdata/edit
```
