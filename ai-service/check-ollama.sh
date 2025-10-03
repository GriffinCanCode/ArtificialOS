#!/bin/bash
# Check Ollama performance settings

echo "Current Ollama models:"
ollama list

echo -e "\nOllama info:"
curl -s http://localhost:11434/api/tags | python3 -m json.tool

echo -e "\nRecommended: Pull Q4 version for speed"
echo "ollama pull gpt-oss:20b-q4"
