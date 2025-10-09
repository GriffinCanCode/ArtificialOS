package sandbox

import (
	"strings"
	"sync"
)

// DOM provides a lightweight document proxy for sandboxed JavaScript
type DOM struct {
	root    *Element
	changes []DOMChange
	mu      sync.RWMutex
}

// Element represents a DOM element
type Element struct {
	TagName     string
	ID          string
	ClassName   string
	TextContent string
	Attributes  map[string]string
	Children    []*Element
	Parent      *Element
}

// NewDOM creates a DOM from HTML structure
func NewDOM() *DOM {
	return &DOM{
		root: &Element{
			TagName:    "document",
			Attributes: make(map[string]string),
			Children:   []*Element{},
		},
		changes: []DOMChange{},
	}
}

// Query finds elements by selector (simplified)
func (d *DOM) Query(selector string) []*Element {
	d.mu.RLock()
	defer d.mu.RUnlock()

	// Simple selector parsing
	if strings.HasPrefix(selector, "#") {
		// ID selector
		id := strings.TrimPrefix(selector, "#")
		if elem := d.findByID(d.root, id); elem != nil {
			return []*Element{elem}
		}
	} else if strings.HasPrefix(selector, ".") {
		// Class selector
		class := strings.TrimPrefix(selector, ".")
		return d.findByClass(d.root, class)
	} else {
		// Tag selector
		return d.findByTag(d.root, selector)
	}

	return []*Element{}
}

// GetChanges returns accumulated DOM changes
func (d *DOM) GetChanges() []DOMChange {
	d.mu.RLock()
	defer d.mu.RUnlock()
	return append([]DOMChange{}, d.changes...)
}

// RecordChange adds a DOM change
func (d *DOM) RecordChange(change DOMChange) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.changes = append(d.changes, change)
}

// Element methods

// GetAttribute retrieves attribute value
func (e *Element) GetAttribute(name string) string {
	return e.Attributes[name]
}

// SetAttribute sets attribute value and records change
func (e *Element) SetAttribute(name, value string) {
	e.Attributes[name] = value
}

// Helper methods for querying

func (d *DOM) findByID(elem *Element, id string) *Element {
	if elem.ID == id {
		return elem
	}
	for _, child := range elem.Children {
		if found := d.findByID(child, id); found != nil {
			return found
		}
	}
	return nil
}

func (d *DOM) findByClass(elem *Element, class string) []*Element {
	var result []*Element
	if strings.Contains(elem.ClassName, class) {
		result = append(result, elem)
	}
	for _, child := range elem.Children {
		result = append(result, d.findByClass(child, class)...)
	}
	return result
}

func (d *DOM) findByTag(elem *Element, tag string) []*Element {
	var result []*Element
	if strings.EqualFold(elem.TagName, tag) {
		result = append(result, elem)
	}
	for _, child := range elem.Children {
		result = append(result, d.findByTag(child, tag)...)
	}
	return result
}

// AddElement adds a child element
func (e *Element) AddElement(child *Element) {
	child.Parent = e
	e.Children = append(e.Children, child)
}

// Remove removes element from parent
func (e *Element) Remove() {
	if e.Parent == nil {
		return
	}
	children := e.Parent.Children[:0]
	for _, child := range e.Parent.Children {
		if child != e {
			children = append(children, child)
		}
	}
	e.Parent.Children = children
	e.Parent = nil
}
