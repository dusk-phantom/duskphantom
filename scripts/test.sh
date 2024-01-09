#!/bin/bash

# $1 [必须] 为传入的参数,即数据集文件夹
# $2 [可选] 为使用的任务执行脚本,默认为task.sh

task_script=${2:-task.sh}

cp $task_script $1/

# 启动镜像挂载指定数据文件夹
docker run --name tmp -v $1:/app cpci
# 执行docker rm tmp但是不输出显示,而是把标准输出重定向到/dev/null
docker rm tmp >/dev/null
