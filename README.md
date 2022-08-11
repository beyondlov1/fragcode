# FragCode

## description
用简写记录代码片段, 用于快速搜索, 复制   

使用技术:```Vue3 + tauri + sqlite ```

## feature
- 添加
```
add fc hello, fragcode
```
- 搜索
```
fc
```
- 占位符替换
```
add fcc hello, $1, $2
fcc fragcode world

-> hello, fragcode, world

just like shell
```
- 复制
```
Press Enter/Return
```
- 清空
```
Ctrl+z
```
- 快速显示/隐藏快捷键
```
Ctrl+Space -> 显示/隐藏
Escape -> 隐藏
```

## build
```
npm run tauri build
```

## TODO
- 如果能做到 rofi 那样就好了
