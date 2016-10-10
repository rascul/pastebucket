#!/bin/sh

: ${paste_url:=https://p.rascul.xyz}

if [ "$1" = "-h" ]; then
	cat <<- EOF
		Send a something to paste bucket at $paste_url.
		Usage:
		        <command> | $0
		        $0 < <file>
		EOF
else
	curl -F 'paste=<-' $paste_url < /dev/stdin
fi
