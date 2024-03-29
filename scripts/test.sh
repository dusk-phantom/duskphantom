#!/bin/bash
# $1 [必须] 为传入的参数,即数据集文件夹
# $2 [可选] 为使用的任务执行脚本,默认为task.sh
task_script=${2:-./scripts/task.sh}
cp $task_script ./data/$1/task.sh
# 启动镜像挂载指定数据文件夹
# docker run --name tmp -v ./data/$1:/app cpci
# 修改上述脚本,指定docker run的时候执行 ./data/$1/task.sh
# 根据实践构建临时container名 如 cpci-yyyy-mm-dd-hh-mm-ss
container_name=cpci-$(date +%Y-%m-%d-%H-%M-%S)
docker run --name $container_name -v ./data/$1:/app cpci ./task.sh
# 执行docker rm tmp但是不输出显示,而是把标准输出重定向到/dev/null
docker rm $container_name > /dev/null
