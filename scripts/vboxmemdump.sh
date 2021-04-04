#!/bin/bash
#Simple script for VirtuaBox memory extraction
# Usage: vboxmemdump.sh <VM name>

VBoxManage debugvm $1 dumpvmcore --filename=$1.elf

size=0x$(objdump -h $1.elf|egrep -w "(Idx|load1)" | tr -s " " |  cut -d " " -f 4)
off=0x$(echo "obase=16;ibase=16;`objdump -h $1.elf|egrep -w "(Idx|load1)" | tr -s " " |  cut -d " " -f 7 | tr /a-z/ /A-Z/`" | bc)
head -c $(($size+$off)) $1.elf|tail -c +$(($off+1)) > $1.raw

rm $1.elf
