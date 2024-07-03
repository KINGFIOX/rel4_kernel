#!/usr/bin/env python3
#
# Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
#
# SPDX-License-Identifier: BSD-2-Clause
#

import subprocess
import sys
import argparse
import time
import os
import shutil
from pygments import highlight
from pygments.lexers import BashLexer
from pygments.formatters import TerminalFormatter

build_dir = "./build"

def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('-b', '--baseline', dest="baseline", action="store_true",
                        help="baseline switch")
    parser.add_argument('-c', '--cpu', dest="cpu_nums", type=int,
                        help="kernel & qemu cpu nums", default=1)
    # parser.add_argument('-a','--arch', dest="arch",type=str,
    #                     help="kernel arch",default="riscv64imac-unknown-none-elf")
    parser.add_argument('-a','--arch', dest="arch",type=str,
                        help="kernel arch",default="aarch64-unknown-linux-gnu")
    args = parser.parse_args()
    return args

def exec_shell(shell_command):
    ret_code = os.system(shell_command)
    return ret_code == 0

def clean_config():
    shell_command = "cd ../kernel && git checkout 5ddda4ebdae7523830259d5b1f676d25d5de125b"
    exec_shell(shell_command)

if __name__ == "__main__":
    args = parse_args()
    clean_config()
    progname = sys.argv[0]
    
    if args.arch != "riscv64imac-unknown-none-elf" and args.arch != "aarch64-unknown-linux-gnu":
        clean_config()
        sys.exit(-1)
    
    print("target is:" + args.arch)
    
    if os.path.exists(build_dir):
        shutil.rmtree(build_dir)
    os.makedirs(build_dir)
    if args.baseline == True:
        shell_command = "cd ../kernel && git checkout baseline"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
    else:
        if args.cpu_nums > 1:
            if not exec_shell("cargo build --release --target " + args.arch + " --features ENABLE_SMP"):
                clean_config()
                sys.exit(-1)
        else:
            if not exec_shell("cargo build --release --target " + args.arch):
                clean_config()
                sys.exit(-1)
    
    if args.cpu_nums > 1:
        shell_command = ""
        if args.arch == "riscv64imac-unknown-none-elf":
	        shell_command = "cd ./build && ../../init-build.sh  -DPLATFORM=spike -DSIMULATION=TRUE -DSMP=TRUE && ninja"
        elif args.arch == "aarch64-unknown-linux-gnu":
            shell_command = "cd ./build && ../../init-build.sh  -DPLATFORM=qemu-arm-virt -DSIMULATION=TRUE -DSMP=TRUE && ninja"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
        sys.exit(0)
    shell_command = ""
    if args.arch == "riscv64imac-unknown-none-elf":
        shell_command = "cd ./build && ../../init-build.sh  -DPLATFORM=spike -DSIMULATION=TRUE && ninja"
    elif args.arch == "aarch64-unknown-linux-gnu":
        shell_command = "cd ./build && ../../init-build.sh  -DPLATFORM=qemu-arm-virt -DSIMULATION=TRUE && ninja"
    if not exec_shell(shell_command):
        clean_config()
        sys.exit(-1)
    clean_config()
