欢迎阅读 Rune 脚本语言指南，希望这些文档可以帮你熟悉 Rune。

在 **回归终端** 中，题目使用 Rune 脚本来为选手的动态环境设置环境变量，以及对选手的提交内容进行检验。受益于此，回归终端可以提供非常灵活的题目设计方式。

Rune 语言是一门开源的脚本语言，不是我们编写实现的，作者在这里：[Rune-rs: GitHub](https://github.com/rune-rs)。本文档主要介绍了 Rune 语言的基本语法和一些常用的内置函数，同时介绍回归终端所提供的一些专有 API 与语言模块。如果你想更加深入的了解这门语言，你可以阅读官方提供的资料：[Rune-rs: Rust Docs](https://rune-rs.github.io/api/rune/)，[Rune-rs: Rune Docs](https://rune-rs.github.io/docs/) 和 [《The Rune Programming Language》](https://rune-rs.github.io/book/)。

Rune 的目标是将 Rust 重新实现为动态编程语言。Rune 的语法和 Rust 十分相似，在动态脚本语言环境中，某些事情的处理方式会有所不同，所以一些区别是不可避免的。

Rune 也缺乏通过类型来保证程序正确性的功能。这样做的好处是能够更快的编译脚本，提供更加易用的[鸭子类型](https://zh.wikipedia.org/zh-cn/%E9%B8%AD%E5%AD%90%E7%B1%BB%E5%9E%8B)，以及方便编写更简洁和紧凑的代码。

在接下来的章节里，我们会介绍 Rune 的基本语法，以及一些常见的概念。如果你之前接触过编程语言，那么你应该很快就能熟悉这些内容。如果你之前写过 Rust 代码，那么你几乎可以直接上手 ———— 大部分时候只需要从 Rust 中删除类型声明就能跑了。
