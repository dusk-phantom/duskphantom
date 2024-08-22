#!/bin/bash
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

# $1 [必须] 为传入的参数,即数据集文件夹
# $2 [可选] 为使用的任务执行脚本,默认为task.sh
task_script=${2:-./scripts/task.sh}
cp $task_script ./data/$1/task.sh
# 启动镜像挂载指定数据文件夹
# docker run --name tmp -v ./data/$1:/app cpci
# 修改上述脚本,指定docker run的时候执行 ./data/$1/task.sh
# 根据实践构建临时container名 如 cpci-yyyy-mm-dd-hh-mm-ss
container_name=cpci-$(date +%Y-%m-%d-%H-%M-%S)
docker run -it --name $container_name -v ./data/$1:/app cpci ./task.sh
# 执行docker rm tmp但是不输出显示,而是把标准输出重定向到/dev/null
docker rm $container_name > /dev/null
