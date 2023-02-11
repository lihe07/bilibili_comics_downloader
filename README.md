# bcdown

[BiliBili漫画](https://manga.bilibili.com/)下载器 使用[Rust](rust-lang.org/)语言实现

之前的[Python版本](https://github.com/lihe07/bilibili-manga-downloader)现在由[Armo00](https://github.com/Armo00)大佬负责维护了

## 特性

1. 异步高性能

   还得是Rust

2. 更加灵活的登录

   工具支持 **扫码登录** 及 **SESSDATA登录** 两种登录方式，且能保存登录结果，无需每次重复登录

3. 更完善的缓存功能

   工具会缓存更多的内容以减少网络请求，缓存地址可配置，默认选取用户的*文档*文件夹

4. 更灵活的导出

   支持分话导出和合并导出，会自动添加Kindle等阅读器可识别的书签

   目前支持**epub**，**pdf**，**zip** 和 **plain** 四种格式

5. 自我更新

   不用在手动替换可执行文件了，现在可以用 `bcdown update`自动更新

6. 给Windows系统做的图形化界面

   在 `0.3.0` 版本之后加入，基于 [native-windows-gui](https://crates.io/crates/native-windows-gui)

## 使用方法（命令行）

工具共有如下几个命令：

- `bcdown info ` - 显示配置信息，缓存大小，登录状态等

- `bcdown login`  - 登录B漫账号，通过二维码或者sessdata

  使用示例：

  - `bcdown login -s [获取到的SESSDATA]` 通过sessdata登录

  - `bcdown login -q` 通过扫描二维码登录
  
    > **备注**：二维码必须用Bilibili客户端扫描
  
- `bcdown clean` - 清空缓存文件夹

- `bcdown search [链接或ID]` - 搜索某个漫画，列出它的全部章节

- `bcdown list` - 列出缓存中的漫画

- `bcdown fetch [链接或ID] <--range [开始]-[结束],[开始]-,-[结束]>` - 将一个漫画下载到本地

  使用示例：

  - `bcdown fetch mc29911 --range 1-20,40-50,60- ` 下载 *mc29911* 第1话到第20话，第40话到第50话 和 第60话之后的所有到本地

- `bcdown export [链接或ID] --format [epub或pdf] <--range [开始]-[结束],[开始]-,-[结束]> <-s 单独导出每一话> <--output [输出位置]> <-g [组大小>] `  - 导出一个本地漫画

 `bcdown update` - 自我更新

## 安装

命令行版本和图形化版本装一个即可，两个都装也没问题，会共享一个配置文件

- 命令行版本
  - 使用cargo：`cargo install bcdown` 或 `cargo binstall bcdown`
  - [下载 Releases](https://github.com/lihe07/bilibili_comics_downloader/releases)
- 图形化版本
  - [下载便携式程序（Windows）](https://www.bilibili.com/video/BV1GJ411x7h7)

## Kindle使用指南

1. 使用 `bcdown export XXXX -s -f pdf` 为每一话创建一个pdf

   > 切换起来会比较麻烦

2. 使用 `bcdown export XXXX -f epub` 导出一个epub文件，之后使用 `ebook-convert`, `Neat Converter` 等工具转换成 azw3 格式（这里推荐[Neat Converter](https://www.neat-reader.cn/downloads/converter)，对大文件支持比较好）

   > azw3往往拥有更好的压缩率，同一个漫画 pdf(1.5G) > epub(300M) > azw3(150M)

3. 使用 `bcdown export XXXX -f plain` 导出为图片文件夹格式，之后使用 `Kindle Comic Converter` 等工具转换成 mobi 或 azw3 格式

## 贡献

请参考[CONTRIBUTING.md](CONTRIBUTING.md)

## 联系方式

- Email: *li@imlihe.com*
- QQ：*3525904273*

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
- 0.3.0 - 增强下载稳定性，修复下载速度统计，优化了pdf和epub的导出，添加了plain导出选项，GUI版本开始制作

## 补充

- 本项目不涉及解锁、交易等功能

- 如果发现了Bug，欢迎创建Issue

- 这个工具的更新会很频繁，建议保持使用最新版

- 短时间内大量的网络请求可能会对B漫造成一定负载。如果网络带宽极大（例如1000Mbps甚至900Mbps），请限制同时下载数量

- 目前没有被封禁的案例，如果您被封禁了，请开一个issue分享下经验

