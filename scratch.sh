#!/bin/bash
trap 'echo "Screensaver launched!"' USR1
echo "Waiting for signal. My PID is $$"
(sleep 2; kill -USR1 $$) &
sleep 5
echo "Done"
