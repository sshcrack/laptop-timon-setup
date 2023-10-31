#!/bin/bash

if [[ $(/usr/bin/id -u) -ne 0 ]]; then
    echo "Not running as root"
    exit
fi


apt install wireguard -y
# openssl enc -pbkdf2 -aes-256-cbc -in wg-key.txt -out wg-key.enc

wg_key_url="https://github.com/sshcrack/laptop-timon-setup/raw/master/wg-key.enc"
wings_token_url="https://github.com/sshcrack/laptop-timon-setup/raw/master/wings_token.enc"
wget $wg_key_url -o wg-key.enc
wget $wings_token_url -o wings_token.enc

openssl enc -d -pbkdf2 -aes-256-cbc -in wg-key.enc -out wg-key.dec
openssl enc -d -pbkdf2 -aes-256-cbc -in wings_token.enc -out wings_token.dec
mv wg-key.dec /etc/wireguard/wg0.conf

token=$(cat wings_token.dec)

wg-quick up wg0
sed -i "10.6.0.16  panel.local" /etc/hosts

curl -sSL https://get.docker.com/ | CHANNEL=stable bash
systemctl enable --now docker

mkdir -p /etc/pterodactyl
curl -L -o /usr/local/bin/wings "https://github.com/pterodactyl/wings/releases/latest/download/wings_linux_$([[ "$(uname -m)" == "x86_64" ]] && echo "amd64" || echo "arm64")"
chmod u+x /usr/local/bin/wings
cd /etc/pterodactyl && sudo wings configure --panel-url http://panel.local --token $token --node 2
