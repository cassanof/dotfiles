# cat /usr/local/bin/msfconsole 
#!/bin/sh
trap "" TSTP
/usr/bin/msfconsole "$@"
