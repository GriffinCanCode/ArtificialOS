"""
Prompt Builder
Centralized prompt construction with deduplication.
"""

class PromptBuilder:
    """Builds prompts from message history."""

    @staticmethod
    def build_conversation(
        system_prompt: str,
        messages: list[tuple[str, str]],
        user_input: str
    ) -> str:
        """
        Build conversation prompt.

        Args:
            system_prompt: System instructions
            messages: List of (role, content) tuples
            user_input: Current user message

        Returns:
            Complete prompt string
        """
        parts = [system_prompt, ""]

        for role, content in messages:
            label = "User" if role == "user" else "Assistant"
            parts.append(f"{label}: {content}")

        parts.append(f"User: {user_input}")

        return "\n\n".join(parts)

    @staticmethod
    def build_structured(
        system: str,
        context: str,
        tools: str,
        request: str
    ) -> str:
        """
        Build structured prompt for UI generation.

        Args:
            system: System instructions
            context: Additional context
            tools: Available tools description
            request: User request

        Returns:
            Complete prompt
        """
        parts = [system]

        if tools:
            parts.append(f"\n=== AVAILABLE TOOLS ===\n{tools}")

        if context:
            parts.append(f"\n=== CONTEXT ===\n{context}")

        parts.append(f"\n=== REQUEST ===\n{request}")

        return "\n".join(parts)

