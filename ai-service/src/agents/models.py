"""UI Data Models."""

from typing import Any
from pydantic import BaseModel, Field


class BlueprintComponent(BaseModel):
    """UI component specification."""

    type: str = Field(..., description="Component type")
    id: str | None = Field(default=None, description="Unique identifier")
    props: dict[str, Any] = Field(default_factory=dict)
    children: list["BlueprintComponent"] = Field(default_factory=list)
    on_event: dict[str, str] | None = Field(default=None)


class Blueprint(BaseModel):
    """Complete UI specification."""

    type: str = Field(default="app")
    title: str
    layout: str = Field(default="vertical")
    components: list[BlueprintComponent] = Field(default_factory=list)
    style: dict[str, Any] = Field(default_factory=dict)
    services: list[str] = Field(default_factory=list)
    service_bindings: dict[str, str] = Field(default_factory=dict)
    lifecycle_hooks: dict[str, list[str]] = Field(default_factory=dict)


BlueprintComponent.model_rebuild()
