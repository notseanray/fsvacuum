#!/bin/bash
cargo build --release
echo "copying into /usr/local/bin directory"
echo "if you wish for another location, copy ./target/release/fsvacuum to somewhere in the path"
sleep 1
su="doas"
sudo -V
if [ $? -eq 0 ]; then
	echo "found sudo, using sudo to copy"
	su="sudo"
else
	echo "failed to find sudo, using doas"
fi
$su cp target/release/fsvacuum /usr/local/bin/
