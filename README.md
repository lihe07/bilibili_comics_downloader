# 暂停维护

由于近期B漫多次更新，加入了新的加密和风控，工具已经无法使用。

目前的所有三方下载器均已失效，继续更新风险较大。

相关Issues:

- [lanyeeee/bilibili-manga-downloader #68](https://github.com/lanyeeee/bilibili-manga-downloader/issues/68)
- [lanyeeee/bilibili-manga-download-script #4](https://github.com/lanyeeee/bilibili-manga-download-script/issues/4)
- [Zeal-L/BiliBili-Manga-Downloader #170](https://github.com/Zeal-L/BiliBili-Manga-Downloader/issues/170)

更多细节也欢迎联系我 (QQ: 3525904273)

# bcdown

[Bilibili漫画](manga.bilibili.com/)下载器，written in [Rust](rust-lang.org/)

这个项目用来接替[之前的python版](https://github.com/lihe07/bilibili-manga-downloader)

## 特性

1. 异步高性能

   网络相关操作使用了基于`tokio`的`reqwests`，较多线程版有较大的性能提升

2. 更加灵活的登录

   工具支持 **扫码登录** 及 **SESSDATA登录** 两种登录方式，且能保存登录结果，无需每次重复登录

3. 更完善的缓存功能

   工具会缓存更多的内容以减少网络请求，缓存地址可配置，默认选取用户的*文档*文件夹

4. 更灵活的导出

   支持分话导出和合并导出，会自动添加Kindle等阅读器可识别的书签

   目前支持**epub**，**pdf**和**zip**三种格式

## 使用方法

工具共有如下几个命令：

- `bcdown info` - 显示配置信息，缓存大小，登录状态

  使用示例：

  - `bcdown info`

    ```
    ℹ 加载配置文件：D:\Documents\bcdown/config.toml
    ℹ bcdown 版本: 0.2.1
    ℹ 登录信息有效！
    ℹ 用户名：[数据删除]
    ℹ 漫币余额：[数据删除]
    ℹ 缓存目录：D:\Documents\bcdown/cache
    ℹ 缓存目录大小：4.629214677959681 GB
    ℹ 默认下载目录：D:\Download
    ```

- `bcdown login`  - 登录B漫账号，通过二维码或者sessdata

  使用示例：

  - `bcdown login -s [数据删除]` 通过sessdata登录

    ```
    ℹ 加载配置文件：D:\Documents\bcdown/config.toml
    ℹ 登录信息有效！
    ℹ 用户名：[数据删除]
    ℹ 漫币余额：[数据删除]
    ```

  - `bcdown login -q` 通过扫描二维码登录

    ```
    ℹ 加载配置文件：D:\Documents\bcdown/config.toml
        ▀ ██▀▀▀▄  ▀ ▄▄ █▄ ▀▄▄ ▀█▀█▄▀▄ ██▀▀█▄
          ▀▀ ▄▀ ▀   █ ▀ ▄ ▄▄▄▄▀▄█▀▀██ █ ▄█▀▀█
           █ █▀█▄▄▀██▄  ▄▀▄▄▄▄▀▄▀▀▀▀█▀▀█▀ ▀▀▀
         █▄█▀█▀██ ▄   ██▄ █▄ ██ ▄ ▄▀█▄ ██▀ ██
        █▀ ▀▄▀▀█▀  █▀▄█▄ ▀█▄▄▄▀█▀▀▀ ▄▀▀▄▀ █▄
         █   █▀█  █▄█  █▄   █▄▄ ▄▀ ██▄█▄  ▀▀█
        ▄ ▄▄▀▄▀ █▄▄▄ ▄▄▀▄██▄█▄▀█▀▀▀▀▄█▀ ▀█▀
        █▀█   ▀▀▄███▀█▄▀█ ▀ ▀▄▀▄█ ██ ████▀ █▀
        ▄▄ █  ▀ █▄▄▀█▀█ ▄█▄▄▄ ▄▄▄█▀▀▄█▀█▀ █▀▀
        █ ▄▄▀▄▀▄██▀ █ █▀ ▀  ▀▄█▄▀ ▄██▄▀█  ▀██
        ▀ ▀▀▀ ▀ ▄▀▄▄▀ ▀▀ █▄██▄▀▄▀▀▀ █▀▀▀██▀▀
        █▀▀▀▀▀█ ▄▀   █▀██▀▀▄█▄▀ ▄ ▀ █ ▀ █▄▀█▀
        █ ███ █ █  ▄▄▄█▀▄▄▀█▀ ▄█▄█▀▀███▀█▄▀▀▄
        █ ▀▀▀ █ ▀ ▀▀▀▄ ▀█ █  ██▄█ ▀▀▄▀  ▄  ▀█
        ▀▀▀▀▀▀▀ ▀▀▀    ▀▀▀▀▀    ▀▀▀▀ ▀▀ ▀ ▀▀▀
    
    
    ✔ 二维码已生成，请扫描二维码登录
    ℹ 如果显示错误，请手动访问：[链接省略]
    ⠇ 等待确认...
    ```

    > **备注**：建议在支持色彩和符号终端中执行，如*Windows Terminal*

- `bcdown clear` - 清空缓存文件夹

- `bcdown search [链接或ID]` - 搜索某个漫画，列出它的全部章节

- `bcdown list` - 列出缓存中的漫画

- `bcdown fetch [链接或ID] <--range [开始]-[结束],[开始]-,-[结束]>` - 将一个漫画下载到本地

  使用示例：

  - `bcdown fetch mc29911 --range 1-20,40-50,60-` 下载 *mc29911* 第1话到第20话，第40话到第50话 和 第60话之后的所有到本地

    ``

- `bcdown export [链接或ID] --format [epub或pdf] <--range [开始]-[结束],[开始]-,-[结束]> <-s 单独导出每一话> <--output [输出位置]> <-g [组大小>]`  - 导出一个本地漫画

## 构建，编译，安装

和大部分rust crates一样，只需clone该存储库，之后执行`cargo build --release` 即可本地构建

> **备注**：鉴于依赖项`printpdf`的特性，只有在添加`--release`标签后，工具才会对PDF执行压缩

这个项目已经发布到`crates.io`上了，因此可以通过`cargo install bcdown`来安装

如果只是普通用户，可以下载编译好的可执行文件：[Releases](https://github.com/lihe07/bilibili_comics_downloader/releases)

## Kindle使用指南

由于kindle阅读器暂时不支持epub格式的电子书，而pdf格式又过于庞大，不便于传输，这里有几种常见解决方案：

1. 使用 `bcdown export XXXX -s -f pdf` 分话导出较小的pdf文件（编码较慢）

   > ​ 在存储空间较小的设备上需要较多次的传输

2. 使用 `bcdown export XXXX -f epub` 导出一个大的epub文件（比pdf快），之后使用 `ebook-convert`, `NeatConverter` 等工具转换成 azw3 格式（这里推荐Neat Converter，对大文件支持比较好）

   > ​ 实测azw3拥有更好的压缩率，pdf(1.5G) > epub(300M) > azw3(150M)
   >
   > ​ 由于不同的导出工具的区别，一些小功能可能会失效（如封面图片，标题等）图片和目录的显示已经经过测试，不会出现较大的问题
   >
## 提交PR

看[CONTRIBUTING.md](CONTRIBUTING.md)

## 联系方式

我的QQ：*3525904273*

## 更新记录

- 0.1.0 - 第一个版本，支持导出PDF
- 0.1.1 - 修复了bug，支持导出epub格式
- 0.1.2 - 修复bug
- 0.1.3 - 修复bug
- 0.1.4 - 修复bug，优化网络模块
- 0.1.5 - 加入导出zip文件的支持，修复bug，优化任务处理
- 0.1.6 - 删除书签中的ORD编码，修复epub中的格式问题
- 0.1.7 - 修复bug，修改压缩方式为Stored
- 0.2.0 - 范围下载/导出，优化网络，加入分组导出功能，升级依赖项
- 0.2.1 - 修复bug
- 0.2.2 - 修复bug

## 补充

这个工具只是个爬虫，不能下载没有解锁的漫画，因此解决不了钞能力的问题（

如果发现了Bug（~~可能~~会有很多，已经发现了不少），欢迎创建Issue

这个工具的更新会很频繁，建议保持使用最新版

由于没有限速功能，短时间内大量的网络请求可能会对B漫造成一定负载。建议不要频繁进行下载操作，尽管目前没有因下载而被封号的案例，但B漫拥有封禁的权力。使用本工具造成的一切损失请自行承担！
