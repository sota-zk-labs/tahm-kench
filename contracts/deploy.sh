#!/bin/bash

# cd to the directory of this script so that this can be run from anywhere
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" || exit 1 ; pwd -P )
cd "$parent_path" || exit 1

# Load environment variables from .env
if [ -f .env ]; then
  # shellcheck disable=SC2046
  # shellcheck disable=SC2002
  export $(cat .env | xargs)
fi

# Check if LOCK_TOKEN_ADDRESS is set
if [ -z "$LOCK_TOKEN_ADDRESS" ]; then
  echo "Error: LOCK_TOKEN_ADDRESS is not set. Please set it in .env or pass it as a one argument.."
  exit 1
fi

# Check if PRIVATE_KEY is set
if [ -z "$PRIVATE_KEY" ]; then
  echo "Error: PRIVATE_KEY is not set. Please set it in .env or pass it as a second argument.."
  exit 1
fi

# Check if RPC_URL is set
if [ -z "$RPC_URL" ]; then
    echo "Error: RPC_URL is not set. Please set it in .env or pass it as a three argument."
    exit 1
fi

# Check if Foundry is installed
if ! [ -x "$(command -v forge)" ]; then
  echo "Error: Foundry (forge) is not installed."
  exit 1
fi

# Compile the contracts
echo "Compiling smart contracts..."
forge build

# Run the deployment script
echo "Deploying Contract..."
forge script script/deployer.s.sol \
  "$LOCK_TOKEN_ADDRESS" \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --sig "run(address _lockToken)"

# Check if deployment was successful
if [ $? -eq 0 ]; then
  echo "Contract successfully deployed!"
else
  echo "Error deploying contract."
  exit 1
fi

