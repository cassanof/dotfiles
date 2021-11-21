#!/bin/sh

# Control variable
# Possible values: NONE, FULL, LOW, CRITICAL
last="NONE"

# Default values for LOW/CRITICAL status
low=20
critical=15

while true; do

  # If battery is plugged, do stuff
  battery="/sys/class/power_supply/BAT0"
  if [ -d $battery ]; then

    capacity=$(cat $battery/capacity)
    status=$(cat $battery/status)

    # If low and discharging
    if [ "$last" != "LOW" ] && [ "$status" = "Discharging" ] && \
       [ $capacity -le $low ]; then
      dunstify "Battery low: $capacity%"
      last=LOW
    fi

    # If critical and discharging
    if [ "$status" = "Discharging" ] && [ $capacity -le $critical ]; then
      dunstify -u critical "Battery very low: $capacity%"
      last=CRITICAL
    fi
  fi
  sleep 60
done
