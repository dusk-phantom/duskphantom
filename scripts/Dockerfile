# 编写一个docker镜像,作用是挂载一个目录,把目录中的c代码
# 全部编译成可执行文件,然后运行

# 以ubuntu 22.04 为基础镜像
FROM ubuntu:23.04

# COPY localtime /etc/localtime
# COPY  sources.list /etc/apt/sources.list

RUN apt-get update -y || true

# 安装 gcc-12跨riscv交叉编译工具链,
# 安装 qemu-user
RUN apt-get install -y  \
    gcc-12-riscv64-linux-gnu \
    qemu-user \
    && apt-get clean


# 挂载目录,目录中应该要有如下三个文件夹
# src: 存放c(sysy)代码
# build: 存放编译后的rv64gc架构可执行文件
# asm: 存放产生的汇编
# log: 存放编译过程产生的编译器日志
# in: 存放输入
# out: 存放输出
# 以上文件夹都是必须的,否则会报错
RUN mkdir /app
WORKDIR /app


# 其中app目录挂在到宿主机的app目录
VOLUME ["/app"]

# 执行脚本
CMD ["/bin/bash" ,"/app/task.sh"]