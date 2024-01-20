#!/bin/bash
# $1 [必须] 为传入的参数,即数据集文件夹
# $2 [可选] 为使用的任务执行脚本,默认为task.sh
# 触碰文件夹sy 中所有文件,刷新最后修改时间
touch ./data/$1/sy/*
./scripts/test.sh $1 $2