# BlockingIO-client

# 実行

1. ビルドする

2. `cargo run`

※ WSL2 + VcXsrv を使う場合は、Native opengl をオフに、Disable access control をオンにする。

# ビルド方法

## Windows
```
> cargo build   # または cargo build --release
```

## macOS

**macOSはOpenGLのバージョンが古いのでサポートできません。** 代わりにWebGLに対応したい。


## Linux

いずれの場合も `gcc` が必要です。

### Ubuntu
```
> sudo apt install libsdl2-dev
> cargo build   # または cargo build --release
```

### Fedora (未検証)
```
> sudo dnf install SDL2-devel
> cargo build   # または cargo build --release
```

### Arch (未検証)
```
> sudo pacman -S sdl2
> cargo build   # または cargo build --release
```

# 設定ファイル

1度実行すると、exeと同じディレクトリに`blocking-io-settings.toml`というファイルが生成される。

* `uuid` - ユーザーID
* `server` - サーバーのアドレス
* `fullscreen` - フルスクリーン

# ログ

環境変数`BLKIO_TRACE=1`を設定すると一番細かいログが出力されるようになる。
