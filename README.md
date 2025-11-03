
```
# テスト環境構築
$ cargo install wasm-pack
$ sudo apt install -y firefox
$ cargo install geckodriver
# テスト実行
$ wasm-pack test --headless --firefox
# ログを出したいとき
```

ログを出したいとき
console::log_1でログを出す。テストが落ちたときだけ表示される。
```
$ GECKODRIVER_LOG=trace wasm-pack test --headless --firefox 
```
