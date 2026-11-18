#!/usr/bin/env bash
# Builds phai-studio.monrak.net and uploads it to the VPS (aaPanel), as the other Phai sites are:
#   bash site/deploy.sh
# The site must exist in aaPanel first (domain phai-studio.monrak.net, PHP 8, its URL rewrite from
# nginx.conf.example). storage/ on the server (the download and visitor counts) is never replaced.
set -euo pipefail

HOST=ubuntu@118.27.149.7
KEY=~/.ssh/phaicity_vps
dest=/www/wwwroot/phai-studio.monrak.net
root=$(cd "$(dirname "$0")/.." && pwd)

python "$root/site/build.py"
name=phai-studio-$(date +%s).tgz
tar --force-local -C "$root/target/site" --exclude=./storage -czf "$root/target/$name" .
scp -i "$KEY" -q "$root/target/$name" "$HOST:/tmp/$name"
rm -f "$root/target/$name"
ssh -i "$KEY" "$HOST" "set -e
    test -d '$dest' || { echo 'No $dest: add the site in aaPanel first' >&2; rm -f /tmp/$name; exit 1; }
    sudo tar -xzf /tmp/$name -C '$dest' --no-same-owner
    sudo mkdir -p '$dest/storage'
    sudo find '$dest' ! -name .user.ini -exec chown www:www {} +   # .user.ini is aaPanel's, locked
    rm -f /tmp/$name
    echo 'uploaded to $dest'"
