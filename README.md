# BlockingIO-client

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

# 実行
```
> cargo run
```

WSL2 + VcXsrv X Server を使う場合は、Native opengl をオフに、Disable access control をオンにする。