# Copyright 2024 Duskphantom Authors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0

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

