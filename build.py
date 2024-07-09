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
    # parser.add_argument('-a', '--arch', dest="architecture", default="riscv64", help="build architecture")
    parser.add_argument('-p', '--platform', dest='platform', default='spike', help="set-platform")
    parser.add_argument('-c', '--cpu', dest="cpu_nums", type=int,
                        help="kernel & qemu cpu nums", default=1)
    args = parser.parse_args()
    return args

def exec_shell(shell_command):
    ret_code = os.system(shell_command)
    return ret_code == 0

def clean_config():
    # shell_command = "cd ../kernel && git checkout 552f173d3d7780b33184ebedefc58329ea5de3ba"
    # exec_shell(shell_command)
    pass

if __name__ == "__main__":
    args = parse_args()
    clean_config()
    progname = sys.argv[0]

    target = ""
    if args.platform == "spike":
        target = "riscv64imac-unknown-none-elf"
    elif args.platform == "qemu-arm-virt":
        target = "aarch64-unknown-none-softfloat"
    
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
            if not exec_shell(f"cargo build --release --target {target} --features ENABLE_SMP"):
                clean_config()
                sys.exit(-1)
        else:
            if not exec_shell(f"cargo build --release --target {target}"):
                clean_config()
                sys.exit(-1)
    
    if args.cpu_nums > 1:
        shell_command = f"cd ./build && ../../init-build.sh  -DPLATFORM={args.platform} -DSIMULATION=TRUE -DSMP=TRUE && ninja"
        if not exec_shell(shell_command):
            clean_config()
            sys.exit(-1)
        sys.exit(0)
    shell_command = f"cd ./build && ../../init-build.sh  -DPLATFORM={args.platform} -DSIMULATION=TRUE && ninja"
    if not exec_shell(shell_command):
        clean_config()
        sys.exit(-1)
    clean_config()
