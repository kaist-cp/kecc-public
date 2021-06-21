#!/usr/bin/env bash

set -e

sshpass -p 'sifive' scp bench-gcc hifiveu:/mnt/home/runner/cs420-final/bench-gcc
sshpass -p 'sifive' ssh hifiveu /usr/sbin/chroot /mnt "chown runner:runner /home/runner/cs420-final/bench-gcc"
sshpass -p 'sifive' ssh hifiveu /usr/sbin/chroot /mnt /bin/su - runner -c "/home/runner/cs420-final/bench-gcc"
