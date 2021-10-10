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

### WSL2 + VcXsrv X Server を使う場合
Native opengl をオフに、Disable access control をオンにする。

## macOS (未検証)
多分 `gcc` が必要
```
> brew install sdl2
> echo 'export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"' >> ~/.bash_profile
> cargo build
```
