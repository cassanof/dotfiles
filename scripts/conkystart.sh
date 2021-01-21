#!/bin/bash

sleep 10 &&
killall conky
sleep 20 &&
conky -c ~/.config/conky/conky.conf

