

[View as English](./README.en.md)

#### 项目描述

本项目通过逆向得到苹果缓存服务器的签名算法，并可以成功注册缓存服务。算法分为两种运行模式。

#### 运行模式

1. **直接运行（x64）**: 效率较高，但只支持64位CPU。已测试可运行在Windows/Linux/macOS上。
2. **模拟器运行**: 兼容性极高，支持所有CPU架构，包括arm64/mips64/riscv64等。速度可能稍慢。

#### 编译方式

- 直接运行: `cargo build --release`
- 模拟运行: `cargo build --release --features=emu`

#### 配置文件

- `cache.json`: 用于设置IP段，与macOS选项相同。
- `mac.toml`: 存储机器码信息，可以通过相关注释在一台新的Mac上使用。注意五码必须合一。

#### 未来计划

1. 通过cxx/uniffi-rs暴露易用的调用接口，支持多种编程语言（如C++/Python/Kotlin/Swift）。
2. 转译相关代码，通过模拟轨迹转换成llvm-ir，然后提升成C代码。