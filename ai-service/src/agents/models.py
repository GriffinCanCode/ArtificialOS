"""UI Data Models."""

from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field


class UIComponent(BaseModel):
    """UI component specification."""
    type: str = Field(..., description="Component type")
    id: str = Field(..., description="Unique identifier")
    props: Dict[str, Any] = Field(default_factory=dict)
    children: List["UIComponent"] = Field(default_factory=list)
    on_event: Optional[Dict[str, str]] = Field(default=None)


class UISpec(BaseModel):
    """Complete UI specification."""
    type: str = Field(default="app")
    title: str
    layout: str = Field(default="vertical")
    components: List[UIComponent] = Field(default_factory=list)
    style: Dict[str, Any] = Field(default_factory=dict)
    services: List[str] = Field(default_factory=list)
    service_bindings: Dict[str, str] = Field(default_factory=dict)
    lifecycle_hooks: Dict[str, List[str]] = Field(default_factory=dict)


UIComponent.model_rebuild()

