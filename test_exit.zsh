trap 'tput civis; sleep 1; tput cnorm' USR1
echo "My PID: $$"
(sleep 2; kill -USR1 $$; exec 1>&-; exec 2>&-) &
sleep 5
