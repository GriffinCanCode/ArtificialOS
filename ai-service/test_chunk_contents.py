#!/usr/bin/env python3
"""Investigate what's in the stream chunks."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "src"))

from langchain_ollama import ChatOllama

print("="*80)
print("INVESTIGATE: What's in the stream chunks?")
print("="*80 + "\n")

llm = ChatOllama(
    model="gpt-oss:20b",
    base_url="http://localhost:11434",
    temperature=0.3,
    num_predict=500,
)

prompt = "Return JSON: {\"test\": 123}"

print("Streaming response...")
print("-"*80)

content_via_attr = ""
chunk_count = 0

for chunk in llm.stream(prompt):
    chunk_count += 1
    
    # Show first 3 chunks and any chunks with content
    if chunk_count <= 3 or (hasattr(chunk, 'content') and chunk.content):  # Show first 3 chunks in detail
        print(f"\nChunk {chunk_count}:")
        print(f"  Type: {type(chunk)}")
        print(f"  Has .content: {hasattr(chunk, 'content')}")
        print(f"  Has .text: {hasattr(chunk, 'text')}")
        
        if hasattr(chunk, 'content'):
            print(f"  .content value: '{chunk.content}'")
            print(f"  .content type: {type(chunk.content)}")
        
        if hasattr(chunk, 'text') and callable(chunk.text):
            print(f"  .text() value: '{chunk.text()}'")
        
        # Show all attributes
        attrs = [attr for attr in dir(chunk) if not attr.startswith('_')]
        print(f"  Attributes: {attrs[:10]}...")  # First 10
    
    # Try accumulating
    if hasattr(chunk, 'content') and chunk.content:
        content_via_attr += chunk.content

print(f"\n" + "="*80)
print(f"RESULTS:")
print(f"  Total chunks: {chunk_count}")
print(f"  Accumulated via .content: '{content_via_attr}' ({len(content_via_attr)} chars)")
print("="*80)

if len(content_via_attr) == 0:
    print("\n⚠️  WARNING: All chunks had empty .content!")
    print("This might be a langchain-ollama version issue.")

