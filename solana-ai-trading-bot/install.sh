#!/bin/bash
# filepath: c:\Users\pc\solana-ai-trading-bot\install.sh

set -e

echo "=== AlphaStriker Installation ==="

# Convert all scripts to Unix format
if command -v dos2unix &> /dev/null; then
    find . -type f -name "*.sh" -exec dos2unix {} \;
else
    sudo apt-get update
    sudo apt-get install -y dos2unix
    find . -type f -name "*.sh" -exec dos2unix {} \;
fi

# Install Python 3 and pip
sudo apt-get install -y python3 python3-pip

# Install Node.js 20 and yarn
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
sudo npm install -g yarn

# Install Docker if not present
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
fi

echo "Docker found. Proceeding with image build."

# Clean Docker cache
docker builder prune -af || true

# Build Docker image
echo "Building Docker image 'alphastriker'..."
docker build --no-cache -t alphastriker .

if [ $? -eq 0 ]; then
    echo "Docker image 'alphastriker' built successfully."
else
    echo "Error: Docker image build failed. Please check the Dockerfile and your Docker setup."
    exit 1
fi

echo "Installation script finished. You can now run AlphaStriker using run.sh."

# Install backend dependencies
echo "Installation des dépendances Python du backend..."
cd backend
pip3 install --upgrade pip
pip3 install -r requirements.txt
cd ..

# Install frontend dependencies
echo "Installation des dépendances Node du frontend..."
cd frontend
yarn install
cd ..

echo "=== Installation terminée ==="