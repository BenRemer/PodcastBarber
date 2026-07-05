#!/usr/bin/env bash

# Define the cleanup function that runs when the script exits
cleanup() {
  echo -e "\n[!] Caught exit signal. Stopping Docker containers..."
  docker compose down
  echo "[✔] Containers stopped cleanly."
  exit 0
}

# Trap SIGINT (Ctrl+C) and SIGTERM, and route them to the cleanup function
trap cleanup SIGINT SIGTERM

echo "[*] Building and starting Docker containers in the background..."
docker compose up --build -d

# Check if the build/start was successful before tailing logs
if [ $? -ne 0 ]; then
  echo "[X] Docker compose failed to start. Exiting."
  exit 1
fi

echo "[*] Attaching to logs. Press Ctrl+C to stop and tear down."
# Follow the logs natively
docker compose logs -f

# If the logs exit on their own (e.g., all containers crash), trigger cleanup
cleanup
