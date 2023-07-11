# Spellcast Solver in Rust
![screenshot.png](screenshot.png)

## Sample
[Browser Version](https://y52en.github.io/spellcast_finder_rust/)  
Please be advised that showing the results may take some time, and is limited to a maximum of 2 swaps.

## Downloads
[Downloads](https://github.com/y52en/spellcast_finder_rust/releases/latest)

## Features
- [x] Capable of searching up to 3 swaps
- [x] Supports multithreading
- [x] Dirty code
- [x] Bare minimum, user-unfriendly interface
- [x] Meaningless commit log

## Performance
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
- Windows 11 8GB (i7-8550U, WSL2)
  - 5.03sec

## Requirements
You'll need to have the following installed to build and run the application:
- Rust
- Node.js  

Please note that you may need to install some dependencies for Tauri. For details, please refer to the following issue thread: https://github.com/tauri-apps/tauri/issues/3701

## Building the Application
```bash
npm install
npm run tauri build
```

## credits
Dictionary Source : [https://github.com/jacksonrayhamilton/wordlist-english](https://github.com/jacksonrayhamilton/wordlist-english)