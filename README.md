# spellcast_finder_rust
![screenshot.png](screenshot.png)

## features
- [x] Capable of searching up to 3 swaps
- [x] Supports multithreading
- [x] Dirty code
- [x] Bare minimum, user-unfriendly interface
- [x] Meaningless commit log

## speed
input
```
q w e r t 
y u i o p 
a s d f g 
h j k l z 
x c v b n 
```

- MacBook Air 8GB (M1, 2020)
  - 2.43sec
- Windows 10 8GB (i7-8550U, WSL2)
  - 5.03sec

## requirements
- rust
- Node.js
- perhaps you need to install some dependencies for tauri
  - see https://github.com/tauri-apps/tauri/issues/3701

## building
```bash
npm install
npm run tauri build
```