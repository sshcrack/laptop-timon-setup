#!/bin/bash

if [[ $(/usr/bin/id -u) -ne 0 ]]; then
    echo "Not running as root"
    exit
fi


echo "Installing packages"
apt install wireguard wget openssl -y

echo "Downloading wireguard config / Wings token"
wg_key_url="https://github.com/sshcrack/laptop-timon-setup/raw/master/wg-key.enc"
wings_token_url="https://github.com/sshcrack/laptop-timon-setup/raw/master/wings_token.enc"
curl $wg_key_url -Lo wg-key.enc
curl $wings_token_url -Lo wings_token.enc

# openssl enc -pbkdf2 -aes-256-cbc -in wg-key.txt -out wg-key.enc
openssl enc -d -pbkdf2 -aes-256-cbc -in wg-key.enc -out wg-key.dec
openssl enc -d -pbkdf2 -aes-256-cbc -in wings_token.enc -out wings_token.dec
echo "Copying wireguard config"
mv wg-key.dec /etc/wireguard/wg0.conf

echo "Starting wireguard"
wg-quick up wg0
echo "10.6.0.16  panel.local" >> /etc/hosts

echo "Installing docker"
curl -sSL https://get.docker.com/ | CHANNEL=stable bash
systemctl enable --now docker

mkdir -p /etc/pterodactyl
echo "Installing wings"
curl -L -o /usr/local/bin/wings "https://github.com/pterodactyl/wings/releases/latest/download/wings_linux_$([[ "$(uname -m)" == "x86_64" ]] && echo "amd64" || echo "arm64")"
chmod u+x /usr/local/bin/wings

echo "Setting up wings..."
token=$(cat wings_token.dec)

echo "Token is $token"
cd /etc/pterodactyl && sudo wings configure --panel-url http://panel.local --token $token --node 2
