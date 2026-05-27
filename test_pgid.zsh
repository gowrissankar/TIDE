trap 'echo "USR1 hit, my PGID is $(ps -o pgrp= -p $$)"' USR1
echo "My PID: $$"
(sleep 2; kill -USR1 $$; exit 0) &
sleep 5
