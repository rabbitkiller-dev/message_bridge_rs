# 简介
通过注册Bot机器人, 实现不同平台用户之间的消息进行同步

((TODO: [补上示例图片]))

## 联系方式
- Discord频道: https://discord.gg/jy3tr4Dq
- QQ群：https://jq.qq.com/?_wv=1027&k=D8ymzW7M

欢迎使用和部署, 加入上方联系方式也可以进行体验

## 环境要求
1. 科学上网
2. NodeJs (v14 以上)
3. 配置config.json文件

> CenterOS
> 安装命令参考

1. [Git](https://git-scm.com/download/linux): 命令: `yum install git`
2. NodeJs (v14 以上): 命令自行baidu
3. 全局安装pm2: `npm install -g pm2`
4. [Rust + Cargo](https://forge.rust-lang.org/infra/other-installation-methods.html): 命令: `curl https://sh.rustup.rs -sSf | sh`
5. 配置文件: `cp config.simple.json config.json` 配置说明: CONFIG.md ((TODO: 说明config.json怎么配置))

## 部署方式

> ps: 以上环境请务必自选解决

```shell
> git clone https://github.com/rabbitkiller-dev/message_bridge_rs
> npm install

## 启动 (pm2进程守护)
> npm run build
> pm2 start server.js --name bridge_js
> pm2 start "cargo run" --name bridge_js
```


## 功能情况

((TODO))

### 指令方面
> QQ
- !绑定 [用户名]

用户名输入桥另一侧你自己的账号名, 用于将两边账号关联
- !确认绑定

> Discord
- !绑定 [用户名]

用户名输入桥另一侧你自己的账号名, 用于将两边账号关联
- !确认绑定

### 2.0 遗留项
1. qq群自动审批
2. 桥后台配置界面
2. bot命令搜图
2. bot命令关联qq与dc用户


```
!帮助
!ping
!绑定 qq [qq号]  (dc命令)
!绑定 dc []  (dc命令)
!确认绑定
!解除绑定
!查看绑定状态
!来点[搜图]
!废话生成器
!猜数字游戏

管理员:
!服务器状态
!重启
!查看所有成员绑定关系
!绑定成员关联 [用户名] [用户名]
!解除成员关联 [用户名]

```
 

