#!/bin/bash
# $1 [必须] 为传入的参数,即数据集文件夹
# $2 [可选] 为使用的任务执行脚本,默认为task.sh
task_script=${2:-task.sh}
cp $task_script $1/
# 触碰文件./app/sy 中所有文件,刷新最后修改时间
touch $1/sy/*
./test.sh