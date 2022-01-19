#!/bin/zsh
targetClass=$2
javac $1 && java tester.Main "${targetClass/.class/}"
