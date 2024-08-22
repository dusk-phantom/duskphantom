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


# 用来指导本地构建过程

dir=$1
dir="data/$dir"
compiler=$2
# 如果没有传入编译器,则使用默认编译器
if [ -z $compiler ]; then
    compiler="compiler"
fi

opt=""
# 遍历所有命令行参数判断是否有-a或者--all
for arg in $@; do
    if [ $arg == "-a" ] || [ $arg == "--all" ]; then
        opt="--all"
    fi
done
# 
if [ $opt == "--all" ]; then
    # 编译所有文件
    echo "compiling all files"
    touch $dir/sy/*
else 
    echo "compiling changed files"
fi

cd $dir
pwd



# 输出当前时间到总日志中
echo $(date +%Y-%m-%d\ %H:%M:%S) >> all.log
# 读取time.log文件最后一行,获取最后一次修改文件时间
# 判断sy文件夹,asm文件夹,in文件夹,out文件夹,log文件夹,lib文件夹 是否存在
# 以要检查的文件夹名为参数,实现检查文件夹是否存在,不存在则创建
function check_folder() {
    dir=$1
    # 如果 $dir 文件夹不存在,则创建
    if [ ! -d "./$dir" ]; then
        mkdir $dir
    fi
}
check_folder sy
check_folder asm
check_folder my_asm
check_folder my_logs
check_folder my_out
check_folder my_exec
check_folder exec
check_folder in
check_folder out
check_folder logs
check_folder time
check_folder lib
check_folder llvm

# TODO,检查lib中的函数库是否存在

# 一个shell函数,获取制定$1文件夹参数中的 所有.sy结尾的且修改时间发生了更新的文件
function edited_files() {
    # 获取$1文件夹中所有以$2结尾的文件
    # 以空格分割,并且赋值给files变量
    files=$(find $1 -name "*.$2")
    # 遍历files变量中的所有文件
    for file in $files; do
        # 获取文件名,不包含后缀
        filename=$(basename $file .sy)
        # 获取文件的最后修改时间
        last_edit=$(stat -c %Y $file)
        # 获取文件无后缀名,使用basename工具以外的方式获得
        base_name=$(basename $file .sy)
        # 从 time/$base_name 中读取 上一次修改时间 
        time_path="time/${base_name}.time" 
        # 如果文件不存在,则赋值为0,如果文件为空,则赋值为0
        last_time=$(cat $time_path 2>/dev/null  || echo 0)
        # 如果文件的最后修改时间大于上一次测试时间,则输出
        if [ $last_edit -gt $last_time ]; then
            echo $filename
            # 并且把新修改时间写入对应文件
            echo $last_edit > $time_path
        fi
    done
}

# 遍历sy文件夹中的所有文件,取出基础文件名
# 遍历in文件夹中的所有文件,取出基础文件名,
# 合并两个基础文件名列表,去除重复部分

# 代码编写如下
# 1. 遍历sy文件夹中的所有文件,取出基础文件名
sy_edited_base_names=$(edited_files sy sy)
# 2. 遍历in文件夹中的所有文件,取出基础文件名,
in_edited_base_names=$(edited_files in in)
# 3. 合并两个基础文件名列表,去除重复部分
edited_base_names=$(echo $sy_edited_base_names $in_edited_base_names | tr ' ' '\n' | sort | uniq)

echo "compiling"
for base_name in $edited_base_names; do
    # 记录正在测试的文件名
    echo "Testing $base_name" >> my_logs/log

    # 定义输入和输出文件路径
    input_file="sy/$base_name.sy"
    output_file="my_asm/$base_name.s"
    log_file="my_logs/$base_name.log"
    llvm_file="llvm/$base_name.ll"

    # 使用自定义编译器编译输入文件，并将错误输出到日志文件
    $compiler -S $input_file -o $output_file -l $llvm_file 1>$log_file 2>&1
done
