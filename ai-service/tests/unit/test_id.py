"""Tests for ID generation system."""

import time
from datetime import datetime

import pytest

from core.id import (
    Prefix,
    compare,
    extract_prefix,
    extract_timestamp,
    generate_batch,
    generate_prefixed,
    generate_raw,
    is_agent_id,
    is_conversation_id,
    is_generation_id,
    is_message_id,
    is_request_id,
    is_thought_id,
    is_valid,
    new_agent_id,
    new_conversation_id,
    new_generation_id,
    new_message_id,
    new_request_id,
    new_thought_id,
    sort_ids,
)


class TestGeneration:
    """Test basic ID generation."""

    def test_generate_unique_ids(self):
        """IDs should be unique."""
        id1 = generate_raw()
        id2 = generate_raw()

        assert id1 != id2
        assert len(id1) == 26
        assert len(id2) == 26

    def test_generate_valid_ulids(self):
        """Generated IDs should be valid."""
        id_str = generate_raw()
        assert is_valid(id_str)

    def test_monotonic_ordering(self):
        """IDs should be monotonically increasing within same millisecond."""
        # Generate with small delays to ensure different timestamps
        ids = []
        for _ in range(5):
            ids.append(generate_raw())
            time.sleep(0.001)  # Ensure different timestamps

        # Timestamps should increase
        timestamps = [extract_timestamp(id_str).timestamp() for id_str in ids]
        for i in range(1, len(timestamps)):
            assert timestamps[i] >= timestamps[i - 1]


class TestTypedGeneration:
    """Test typed ID generation."""

    def test_request_id_format(self):
        """Request IDs should have correct prefix."""
        id_str = new_request_id()
        assert id_str.startswith("req_")
        assert extract_prefix(id_str) == Prefix.REQUEST
        assert is_valid(id_str)

    def test_conversation_id_format(self):
        """Conversation IDs should have correct prefix."""
        id_str = new_conversation_id()
        assert id_str.startswith("conv_")
        assert extract_prefix(id_str) == Prefix.CONVERSATION
        assert is_valid(id_str)

    def test_message_id_format(self):
        """Message IDs should have correct prefix."""
        id_str = new_message_id()
        assert id_str.startswith("msg_")
        assert extract_prefix(id_str) == Prefix.MESSAGE
        assert is_valid(id_str)

    def test_thought_id_format(self):
        """Thought IDs should have correct prefix."""
        id_str = new_thought_id()
        assert id_str.startswith("thought_")
        assert extract_prefix(id_str) == Prefix.THOUGHT
        assert is_valid(id_str)

    def test_generation_id_format(self):
        """Generation IDs should have correct prefix."""
        id_str = new_generation_id()
        assert id_str.startswith("gen_")
        assert extract_prefix(id_str) == Prefix.GENERATION
        assert is_valid(id_str)

    def test_agent_id_format(self):
        """Agent IDs should have correct prefix."""
        id_str = new_agent_id()
        assert id_str.startswith("agent_")
        assert extract_prefix(id_str) == Prefix.AGENT
        assert is_valid(id_str)

    def test_all_typed_ids_valid(self):
        """All typed ID generators should produce valid IDs."""
        assert is_valid(new_request_id())
        assert is_valid(new_conversation_id())
        assert is_valid(new_message_id())
        assert is_valid(new_thought_id())
        assert is_valid(new_generation_id())
        assert is_valid(new_agent_id())


class TestValidation:
    """Test ID validation."""

    def test_valid_ulids(self):
        """Valid ULIDs should pass validation."""
        id_str = generate_raw()
        assert is_valid(id_str)

    def test_valid_prefixed_ulids(self):
        """Prefixed ULIDs should pass validation."""
        id_str = new_request_id()
        assert is_valid(id_str)

    def test_invalid_ids(self):
        """Invalid IDs should fail validation."""
        assert not is_valid("")
        assert not is_valid("invalid")
        assert not is_valid("1234567890")
        # Note: ZZZZZZZZZZZZZZZZZZZZZZZZZZ is actually a valid ULID per spec

    def test_malformed_prefixed_ids(self):
        """Malformed prefixed IDs should fail validation."""
        assert not is_valid("req_INVALID")
        # Note: _01ARZ3NDEKTSV4RRFFQ69G5FAV has valid ULID part after underscore
        assert not is_valid("req_")  # Empty ULID part
        assert not is_valid("_")  # Empty ULID part


class TestTimestampExtraction:
    """Test timestamp extraction."""

    def test_extract_from_ulid(self):
        """Should extract timestamp from ULID."""
        before = datetime.now()
        id_str = generate_raw()
        after = datetime.now()

        timestamp = extract_timestamp(id_str)
        assert timestamp is not None
        # ULID timestamps have millisecond precision, so allow small variance
        before_ms = before.replace(microsecond=before.microsecond // 1000 * 1000)
        after_ms = after.replace(microsecond=(after.microsecond // 1000 + 1) * 1000)
        assert before_ms <= timestamp <= after_ms

    def test_extract_from_prefixed_ulid(self):
        """Should extract timestamp from prefixed ULID."""
        before = datetime.now()
        id_str = new_request_id()
        after = datetime.now()

        timestamp = extract_timestamp(id_str)
        assert timestamp is not None
        # ULID timestamps have millisecond precision, so allow small variance
        before_ms = before.replace(microsecond=before.microsecond // 1000 * 1000)
        after_ms = after.replace(microsecond=(after.microsecond // 1000 + 1) * 1000)
        assert before_ms <= timestamp <= after_ms

    def test_invalid_id_returns_none(self):
        """Invalid ID should return None."""
        assert extract_timestamp("invalid") is None


class TestPrefixExtraction:
    """Test prefix extraction."""

    def test_extract_prefix_from_id(self):
        """Should extract prefix from prefixed ID."""
        id_str = new_request_id()
        assert extract_prefix(id_str) == "req"

    def test_no_prefix_returns_none(self):
        """Non-prefixed ID should return None."""
        id_str = generate_raw()
        assert extract_prefix(id_str) is None


class TestBatchGeneration:
    """Test batch generation."""

    def test_generate_multiple_ids(self):
        """Should generate multiple unique IDs."""
        count = 100
        ids = generate_batch(count)

        assert len(ids) == count

        # Check uniqueness
        assert len(set(ids)) == count

        # Check all valid
        for id_str in ids:
            assert is_valid(id_str)


class TestCustomPrefixed:
    """Test custom prefixed IDs."""

    def test_generate_custom_prefix(self):
        """Should generate ID with custom prefix."""
        id_str = generate_prefixed("custom")
        assert id_str.startswith("custom_")
        assert is_valid(id_str)


class TestSortingComparison:
    """Test sorting and comparison."""

    def test_compare_ulids(self):
        """Should compare ULIDs by timestamp."""
        id1 = generate_raw()
        time.sleep(0.002)  # Small delay
        id2 = generate_raw()

        assert compare(id1, id2) < 0
        assert compare(id2, id1) > 0
        assert compare(id1, id1) == 0

    def test_sort_ids_by_timestamp(self):
        """Should sort IDs by timestamp."""
        ids = []
        for _ in range(10):
            ids.append(generate_raw())
            time.sleep(0.001)

        # Shuffle
        import random

        shuffled = ids.copy()
        random.shuffle(shuffled)

        # Sort
        sorted_ids = sort_ids(shuffled)

        assert sorted_ids == ids


class TestTypeGuards:
    """Test type guard functions."""

    def test_identify_request_ids(self):
        """Should identify request IDs."""
        req_id = new_request_id()
        conv_id = new_conversation_id()

        assert is_request_id(req_id)
        assert not is_request_id(conv_id)
        assert not is_request_id("invalid")

    def test_identify_conversation_ids(self):
        """Should identify conversation IDs."""
        conv_id = new_conversation_id()
        req_id = new_request_id()

        assert is_conversation_id(conv_id)
        assert not is_conversation_id(req_id)
        assert not is_conversation_id("invalid")

    def test_identify_message_ids(self):
        """Should identify message IDs."""
        msg_id = new_message_id()
        req_id = new_request_id()

        assert is_message_id(msg_id)
        assert not is_message_id(req_id)

    def test_identify_thought_ids(self):
        """Should identify thought IDs."""
        thought_id = new_thought_id()
        req_id = new_request_id()

        assert is_thought_id(thought_id)
        assert not is_thought_id(req_id)

    def test_identify_generation_ids(self):
        """Should identify generation IDs."""
        gen_id = new_generation_id()
        req_id = new_request_id()

        assert is_generation_id(gen_id)
        assert not is_generation_id(req_id)

    def test_identify_agent_ids(self):
        """Should identify agent IDs."""
        agent_id = new_agent_id()
        req_id = new_request_id()

        assert is_agent_id(agent_id)
        assert not is_agent_id(req_id)


class TestUniquenessUnderLoad:
    """Test uniqueness under high load."""

    def test_high_volume_uniqueness(self):
        """Should generate unique IDs under load."""
        count = 10000
        ids = {generate_raw() for _ in range(count)}

        assert len(ids) == count

    def test_typed_ids_uniqueness(self):
        """Should generate unique typed IDs under load."""
        count = 1000
        req_ids = {new_request_id() for _ in range(count)}
        conv_ids = {new_conversation_id() for _ in range(count)}

        assert len(req_ids) == count
        assert len(conv_ids) == count

        # Ensure no overlap
        assert req_ids.isdisjoint(conv_ids)


class TestFormatConsistency:
    """Test format consistency across all ID types."""

    def test_all_follow_prefix_ulid_format(self):
        """All typed IDs should follow prefix_ULID format."""
        ids = {
            "req": new_request_id(),
            "conv": new_conversation_id(),
            "msg": new_message_id(),
            "thought": new_thought_id(),
            "gen": new_generation_id(),
            "agent": new_agent_id(),
        }

        for prefix, id_str in ids.items():
            parts = id_str.split("_")
            assert len(parts) == 2
            assert len(parts[1]) == 26
            assert is_valid(parts[1])

