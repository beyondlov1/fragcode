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

### TODO
- 想办法快速打开
- 想办法复制后关闭
- 如果能做到 rofi 那样就好了
