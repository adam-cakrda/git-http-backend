# Git-Http-Backend

this is a simple http server for git
it can be used as a backend for git-http-backend

```shell
cargo run --release --bin server -- -r 'e:' -p 80 -a 'localhost'
```

Next, create a folder `test` under drive E.
```shell
mkdir e:\test
```
Then, create a file `test.git` under drive E.
```shell
git init --bare e:\test\test.git
```

Now we can clone the repository
```shell
git clone http://localhost/test/test.git
```