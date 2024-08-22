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
# 触碰文件夹sy 中所有文件,刷新最后修改时间
touch ./data/$1/sy/*
./scripts/test.sh $1 $2