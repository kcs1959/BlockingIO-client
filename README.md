# BlockingIO-client

# 実行

1. <https://github.com/kcs1959/BlockingIO-api>をlocalhostで動かす。または[ここ](https://github.com/kcs1959/BlockingIO-client/blob/feature/socket-io/src/mock_server.rs#L23-L24)を書き換えてサーバーのアドレスを変更する

2. ビルドする

3. `cargo run`

※ WSL2 + VcXsrv を使う場合は、Native opengl をオフに、Disable access control をオンにする。

# ビルド方法

## Windows
```
> cargo build
```

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

## macOS

**macOSはOpenGLのバージョンが古いのでサポートできません。** 代わりにWebGLに対応したい。
