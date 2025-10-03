"""
Context Builder
Intelligent context construction for LLM prompts
"""

import logging
from typing import List, Optional, Dict, Any
from pydantic import BaseModel, Field

from services import Service, ServiceRegistry, ServiceCategory

logger = logging.getLogger(__name__)


class Intent(BaseModel):
    """Parsed user intent"""
    primary_type: str = Field(..., description="App type (notes, todo, chat, etc)")
    required_services: List[str] = Field(default_factory=list)
    complexity: str = Field(default="simple", description="simple|medium|complex")
    features: List[str] = Field(default_factory=list)


class Context(BaseModel):
    """Complete context for LLM prompt"""
    intent: Intent
    services: List[Service]
    templates: List[str] = Field(default_factory=list)
    examples: List[str] = Field(default_factory=list)
    data_models: Dict[str, Dict] = Field(default_factory=dict)


class ContextBuilder:
    """
    Builds intelligent context for UI generation.
    Discovers relevant services and constructs prompts.
    """
    
    # Intent patterns for keyword matching
    PATTERNS = {
        "notes": ["notes", "note", "notebook", "journal"],
        "todo": ["todo", "task", "checklist", "reminder"],
        "chat": ["chat", "message", "conversation", "messenger"],
        "calculator": ["calculator", "calc", "math", "compute"],
        "dashboard": ["dashboard", "overview", "analytics", "stats"],
        "form": ["form", "survey", "questionnaire", "input"],
        "gallery": ["gallery", "photos", "images", "album"],
        "editor": ["editor", "text", "document", "write"],
    }
    
    # Service requirements by app type
    SERVICE_MAP = {
        "notes": ["storage", "ai"],
        "todo": ["storage"],
        "chat": ["storage", "ai"],
        "calculator": [],
        "dashboard": ["storage"],
        "form": ["storage"],
        "gallery": ["storage"],
        "editor": ["storage", "ai"],
    }
    
    def __init__(self, registry: ServiceRegistry):
        self.registry = registry
        logger.info("Context builder initialized")
    
    def analyze(self, request: str) -> Intent:
        """
        Analyze user request to determine intent.
        
        Args:
            request: User's natural language request
            
        Returns:
            Parsed intent
        """
        request_lower = request.lower()
        
        # Match app type
        primary_type = "generic"
        for app_type, keywords in self.PATTERNS.items():
            if any(kw in request_lower for kw in keywords):
                primary_type = app_type
                break
        
        # Determine required services
        required_services = self.SERVICE_MAP.get(primary_type, [])
        
        # Add services based on keywords
        if any(word in request_lower for word in ["sync", "cloud", "realtime"]):
            if "storage" not in required_services:
                required_services.append("storage")
        
        if any(word in request_lower for word in ["ai", "smart", "intelligent"]):
            if "ai" not in required_services:
                required_services.append("ai")
        
        if any(word in request_lower for word in ["auth", "login", "user"]):
            if "auth" not in required_services:
                required_services.append("auth")
        
        # Determine complexity
        complexity = "simple"
        if len(required_services) > 2 or len(request.split()) > 10:
            complexity = "medium"
        if "complex" in request_lower or len(required_services) > 3:
            complexity = "complex"
        
        # Extract features
        features = []
        feature_keywords = {
            "tags": ["tag", "label", "category"],
            "search": ["search", "find", "filter"],
            "export": ["export", "download", "save"],
            "share": ["share", "collaborate", "send"],
        }
        
        for feature, keywords in feature_keywords.items():
            if any(kw in request_lower for kw in keywords):
                features.append(feature)
        
        return Intent(
            primary_type=primary_type,
            required_services=required_services,
            complexity=complexity,
            features=features
        )
    
    def discover_services(self, intent: Intent) -> List[Service]:
        """
        Discover relevant services for intent.
        
        Args:
            intent: Parsed intent
            
        Returns:
            List of relevant services
        """
        services = []
        
        # Get explicitly required services
        for service_id in intent.required_services:
            service = self.registry.get_definition(service_id)
            if service:
                services.append(service)
        
        # Discover additional services if needed
        # (currently manual, can add semantic search later)
        
        return services
    
    def build(self, request: str) -> Context:
        """
        Build complete context for request.
        
        Args:
            request: User's natural language request
            
        Returns:
            Complete context with services, templates, examples
        """
        # Analyze intent
        intent = self.analyze(request)
        logger.info(
            f"Intent: {intent.primary_type}, "
            f"Services: {intent.required_services}, "
            f"Complexity: {intent.complexity}"
        )
        
        # Discover services
        services = self.discover_services(intent)
        
        # Get templates for app type
        templates = self._get_templates(intent)
        
        # Get examples
        examples = self._get_examples(intent)
        
        # Extract data models from services
        data_models = {}
        for service in services:
            for model in service.data_models:
                data_models[model.name] = model.fields
        
        return Context(
            intent=intent,
            services=services,
            templates=templates,
            examples=examples,
            data_models=data_models
        )
    
    def format_prompt(self, context: Context) -> str:
        """
        Format context as string for LLM prompt.
        
        Args:
            context: Built context
            
        Returns:
            Formatted context string
        """
        lines = []
        
        # Add service context
        if context.services:
            service_ids = [s.id for s in context.services]
            lines.append(self.registry.get_context(service_ids))
        
        # Add templates
        if context.templates:
            lines.append("\n=== TEMPLATES ===")
            for template in context.templates:
                lines.append(template)
        
        # Add examples
        if context.examples:
            lines.append("\n=== EXAMPLES ===")
            for example in context.examples:
                lines.append(example)
        
        return "\n".join(lines)
    
    def _get_templates(self, intent: Intent) -> List[str]:
        """Get relevant templates for intent"""
        templates = []
        
        if intent.primary_type == "notes":
            templates.append("""
CRUD Template:
- List view (display all items)
- Detail view (view/edit single item)
- Create button/form
- Delete action
- Update/edit functionality
""")
        
        if intent.primary_type == "todo":
            templates.append("""
Task List Template:
- Input field for new tasks
- List of tasks with checkboxes
- Mark complete/incomplete
- Delete task option
- Filter by status
""")
        
        return templates
    
    def _get_examples(self, intent: Intent) -> List[str]:
        """Get relevant examples for intent"""
        examples = []
        
        if intent.primary_type == "notes":
            examples.append("""
Example: Simple Notes App

Components:
- Text input for note title
- Textarea for note content
- Save button (calls storage.data.save)
- Notes list (loads from storage.data.load)

Service Bindings:
- on_mount: storage.data.load("notes")
- save_btn.click: storage.data.save("notes", data)
""")
        
        return examples

