# 编写一个docker镜像,作用是挂载一个目录,把目录中的c代码
# 全部编译成可执行文件,然后运行

# 以ubuntu 22.04 为基础镜像
FROM ubuntu:23.04

# COPY localtime /etc/localtime
# COPY  sources.list /etc/apt/sources.list

RUN apt-get update -y 

RUN apt-get install -y \
    gcc-12-riscv64-linux-gnu \
    qemu-user \
    clang llvm \
    && apt-get clean \
    && apt-get autoclean 

# 创建挂载目录
RUN mkdir /app
WORKDIR /app
VOLUME ["/app"]

# 执行脚本
CMD ["/bin/bash" ,"/app/task.sh"]