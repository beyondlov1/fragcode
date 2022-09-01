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
- 从其他地方复制到剪切板后, 快速添加
```
cp fg
----------------------
| Clipboard Text ... |
----------------------
input -> Enter / textarea -> Ctrl+Enter
```
- 上下切换选择
```
焦点在输入框时  ArrowUp/ArrowDown
```
- 输入框Tab键快速输入剪切板内容
```
fg <-TAB
-> 
fg CLIPBOARD_TEXT
```

- 列表前5个显示历史剪切板内容, 点击复制

- 增加占位符输入训练, 根据每次输入占位符的值训练正则表达式模型, 并在下次搜索到该条数据的时候, 自动提示(至少训练三次才会有效果)

## build
```
npm run tauri build
```

## TODO
- 如果能做到 rofi 那样就好了
