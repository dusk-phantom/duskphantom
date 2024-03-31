#!/bin/bash

# 示范运行脚本,放到符合规范的数据集下面执行
# echo "task.sh: start"



# 打印任务开始日志信息
function log_start() {
    echo "qemu-riscv64 --version" > version.log
    qemu-riscv64 --version >> version.log
    echo "riscv64-linux-gnu-gcc-12 --version" >> version.log
    riscv64-linux-gnu-gcc-12 --version >> version.log
    # 输出当前时间到总日志中
    echo $(date +%Y-%m-%d\ %H:%M:%S) >> all.log
    echo "task.sh: start"
}
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
function init_project(){
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
}
# 一个shell函数,获取制定$1文件夹参数中的 所有$2结尾的且修改时间发生了更新的文件
function edited_files() {
    # 获取$1文件夹中所有以$2结尾的文件
    # 以空格分割,并且赋值给files变量
    files=$(find $1 -name "*.$2")
    # 遍历files变量中的所有文件
    for file in $files; do
        # 获取文件的最后修改时间
        last_edit=$(stat -c %Y $file)
        # 获取文件无后缀名,使用basename工具以外的方式获得
        base_name=$(basename $file .$2)
        # 从 time/$base_name 中读取 上一次修改时间 
        time_path="time/${base_name}.time" 
        # 如果文件不存在,则赋值为0,如果文件为空,则赋值为0
        last_time=$(cat $time_path 2>/dev/null  || echo 0)
        # 如果文件的最后修改时间大于上一次测试时间,则输出
        if [ $last_edit -gt $last_time ]; then
            echo $base_name
            # 并且把新修改时间写入对应文件
            echo $last_edit > $time_path
        fi
    done
}

function all_edited_files(){
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
    for base_name in $edited_base_names; do
        echo $base_name
    done
}


function gcc_compile(){
    edited_base_names=$@
    # 测试代码
    for base_name in $edited_base_names; do
        echo "test $base_name" >> log
        # 使用 riscv64-linux-gnu-gcc-12 编译 sy/$base_name.sy 文件 为 asm/$base_name.s 文件
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        riscv64-linux-gnu-gcc-12 -x c sy/$base_name.sy -Llib -lsysy -S -o asm/$base_name.s 2>logs/$base_name.log
        # 如果编译成功,则编译 asm/$base_name.s 文件 为 exec/$base_name 文件
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        if [ $? -eq 0 ]; then
            riscv64-linux-gnu-gcc-12 asm/$base_name.s -o exec/$base_name 2>>logs/$base_name.log
        fi
        # 如果编译仍然成功
        # 则判断 in/$base_name.in 文件是否存在,
        # 如果存在,则 使用 qemu-riscv64 执行 exec/$base_name < in/$base_name.in > out/$base_name.out
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        if [ $? -eq 0 ]; then
            if [ -f in/$base_name.in ]; then
                echo "with input"
                qemu-riscv64 -L /usr/riscv64-linux-gnu exec/$base_name <in/$base_name.in >out/$base_name.out 2>>logs/$base_name.log
            else
                echo "without input"
                qemu-riscv64 -L /usr/riscv64-linux-gnu exec/$base_name >out/$base_name.out 2>>logs/$base_name.log
            fi
            # 返回值换行写入到 out/$base_name.out 文件中
            ret=$?  
            echo "" >> out/$base_name.out
            echo $ret >> out/$base_name.out
        fi
    done
}
function compiler_compile(){
    edited_base_names=$@
    # 测试代码
    for base_name in $edited_base_names; do
        echo "test $base_name" >> my_logs/log
        # 使用 riscv64-linux-gnu-gcc-12 编译 sy/$base_name.sy 文件 为 asm/$base_name.s 文件
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        ./compiler -S sy/$base_name.sy -o my_asm/$base_name.s 2>my_logs/$base_name.log
        # 如果编译成功,则编译 asm/$base_name.s 文件 为 exec/$base_name 文件
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        if [ $? -eq 0 ]; then
            riscv64-linux-gnu-gcc-12 my_asm/$base_name.s -o my_exec/$base_name 2>>my_logs/$base_name.log
        fi
        # 如果编译仍然成功
        # 则判断 in/$base_name.in 文件是否存在,
        # 如果存在,则 使用 qemu-riscv64 执行 exec/$base_name < in/$base_name.in > out/$base_name.out
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        if [ $? -eq 0 ]; then
            if [ -f in/$base_name.in ]; then
                echo "with input"
                qemu-riscv64 -L /usr/riscv64-linux-gnu my_exec/$base_name <in/$base_name.in >my_out/$base_name.out 2>>my_logs/$base_name.log
            else
                echo "without input"
                qemu-riscv64 -L /usr/riscv64-linux-gnu my_exec/$base_name >my_out/$base_name.out 2>>my_logs/$base_name.log
            fi
            # 返回值换行写入到 out/$base_name.out 文件中
            ret=$?  
            echo "" >> my_out/$base_name.out
            echo $ret >> my_out/$base_name.out
        fi
    done
}
function compare(){
    edited_base_names=$@
    echo "##################### " >> diff.log
    for base_name in $edited_base_names; do
        echo "test $base_name" >> diff.log
        diff out/$base_name.out my_out/$base_name.out >> diff.log
    done
}
function clean_history(){
    rm -rf asm my_asm 
    rm -rf my_exec exec
    rm -rf my_logs logs my_out 
    rm -rf time
}

# 询问是否清理此前生成文件
read -p "是否清理文件[y/n]:" clean
if [ "$clean" = "y" ]; then
    clean_history
fi

log_start
init_project
to_test_files=$(all_edited_files)
gcc_compile $to_test_files
if [ -f "./compiler" ]; then
    compiler_compile $to_test_files
    compare $to_test_files
fi
echo "task.sh: end"

