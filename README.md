# BlockingIO-client

# 実行

1. ビルドする

2. `cargo run`

※ WSL2 + VcXsrv を使う場合は、Native opengl をオフに、Disable access control をオンにする。

# ビルド方法

## Windows
```
> cargo build
```

## macOS

**macOSはOpenGLのバージョンが古いのでサポートできません。** 代わりにWebGLに対応したい。


## Linux

いずれの場合も `gcc` が必要です。

### Ubuntu
```
> sudo apt install libsdl2-dev
> cargo build
```

### Fedora (未検証)
```
> sudo dnf install SDL2-devel
> cargo build
```

### Arch (未検証)
```
> sudo pacman -S sdl2
> cargo build
```

# サーバーのアドレス

1度実行すると、exeと同じディレクトリに`blocking-io-settings.toml`というファイルが生成されるので、`server`の項目を書き換える
