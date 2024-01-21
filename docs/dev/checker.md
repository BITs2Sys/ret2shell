# 题目验证流程和具体架构设计

## 选手参与流程

1. 选手打开题目，下载题目附件，解出flag，提交flag并进行静态验证；
2. 选手打开题目，下载动态附件，解出flag，提交flag并进行动态映射验证；
3. 选手打开题目，启动题目环境，解出flag，提交flag并与环境中的flag对比验证；

## 管理员管理

1. 上传静态附件，设置flag；
2. 上传动态附件，设置flag；
3. 上传附件，不设置flag，配置题目镜像；

一道题目可以有多个flag，也可以跨模式。

## 设计

文件管理和环境管理不作为插件实现的一部分，文件管理分为静态文件（全提供）和动态文件（映射提供），环境管理方面只有上传镜像与配置镜像关系（类docker compose）。

flag的生成与验证是插件的核心：

1. 当选手提交flag时，flag验证模块会接收用户相关信息，提交内容，题目信息，并配合文件管理和环境管理进行验证；
2. 当选手下载文件时，由平台调用flag验证模块生成对应的flag并存入缓存，接着将文件返回给选手；
3. 当选手启动环境时，由平台调用flag验证模块生成对应的flag并存入缓存，接着启动环境并将连接地址返回给选手；

## trait

```rust
pub trait FlagChecker {
  async fn check(&self, user: &User, challenge: &Challenge, flag: &str) -> Result<bool, CheckerError>;
  async fn flag(&self, user: &User, challenge: &Challenge) -> Result<String, CheckerError>;
  async fn env_vars(&self, user: &User, challenge: &Challenge) -> Result<HashMap<String, String>, CheckerError>;
}

...

pub struct StaticAttachmentChecker {
}

pub struct DynamicAttachmentChecker {
  cache: &RedisPool,
}

pub struct EnvironmentChecker {
  cache: &RedisPool,
}
```
