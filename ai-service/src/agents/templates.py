"""
App Templates
Pre-built templates for common app patterns
"""

from typing import Dict, List
from pydantic import BaseModel


class Template(BaseModel):
    """App template definition"""
    id: str
    name: str
    description: str
    pattern: str
    components: List[str]
    service_requirements: List[str]


class TemplateLibrary:
    """Library of app templates"""
    
    TEMPLATES = {
        "crud": Template(
            id="crud",
            name="CRUD Application",
            description="Create, Read, Update, Delete pattern",
            pattern="""
CRUD Pattern:
1. List View: Display all items
2. Detail View: View/edit single item
3. Create: Form to add new item
4. Update: Edit existing item
5. Delete: Remove item

Layout:
- Header with title
- Create button/form
- List of items
- Item actions (edit, delete)
""",
            components=["list", "form", "button", "input"],
            service_requirements=["storage"]
        ),
        
        "form": Template(
            id="form",
            name="Form Application",
            description="Multi-step form with validation",
            pattern="""
Form Pattern:
1. Input fields with validation
2. Submit button
3. Success/error feedback
4. Data persistence

Layout:
- Form container
- Input fields
- Submit button
- Status message
""",
            components=["form", "input", "button", "text"],
            service_requirements=["storage"]
        ),
        
        "dashboard": Template(
            id="dashboard",
            name="Dashboard",
            description="Data visualization and metrics",
            pattern="""
Dashboard Pattern:
1. Grid layout for cards/widgets
2. Key metrics display
3. Data visualization
4. Refresh capability

Layout:
- Grid container
- Metric cards
- Charts/graphs
- Refresh button
""",
            components=["grid", "card", "chart", "button"],
            service_requirements=["storage"]
        ),
        
        "chat": Template(
            id="chat",
            name="Chat Interface",
            description="Conversational interface",
            pattern="""
Chat Pattern:
1. Message list/history
2. Input field for new message
3. Send button
4. Real-time updates

Layout:
- Message container (scrollable)
- Message bubbles
- Input + send button
- Typing indicator
""",
            components=["list", "input", "button", "text"],
            service_requirements=["storage", "ai"]
        ),
    }
    
    @classmethod
    def get(cls, template_id: str) -> Template:
        """Get template by ID"""
        return cls.TEMPLATES.get(template_id)
    
    @classmethod
    def list_all(cls) -> List[Template]:
        """List all templates"""
        return list(cls.TEMPLATES.values())
    
    @classmethod
    def search(cls, query: str) -> List[Template]:
        """Search templates by query"""
        query_lower = query.lower()
        results = []
        
        for template in cls.TEMPLATES.values():
            if (query_lower in template.name.lower() or
                query_lower in template.description.lower() or
                any(query_lower in req for req in template.service_requirements)):
                results.append(template)
        
        return results

