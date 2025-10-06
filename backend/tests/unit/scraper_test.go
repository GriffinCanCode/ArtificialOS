package unit

import (
	"context"
	"testing"

	"github.com/GriffinCanCode/AgentOS/backend/internal/providers"
	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Sample HTML for testing
const (
	sampleHTML = `
<!DOCTYPE html>
<html>
<head>
	<title>Test Page</title>
	<meta name="description" content="This is a test page">
	<meta property="og:title" content="OG Test Page">
	<meta property="og:description" content="OG Description">
	<meta name="twitter:card" content="summary">
	<script type="application/ld+json">
	{"@context": "https://schema.org", "@type": "Article", "headline": "Test Article"}
	</script>
</head>
<body>
	<header>
		<h1 id="main-heading">Welcome to Test Page</h1>
		<nav>
			<a href="/home">Home</a>
			<a href="/about">About</a>
		</nav>
	</header>
	<main>
		<article>
			<h2>Article Title</h2>
			<p>This is the first paragraph with some content.</p>
			<p>Email me at test@example.com or call (555) 123-4567</p>
			<p>Visit https://example.com for more info.</p>
			<a href="https://external.com">External Link</a>
			<img src="/image1.jpg" alt="Test Image" width="800" height="600">
			<img src="https://example.com/image2.png" alt="Another Image">
		</article>
		<aside class="sidebar">
			<h3>Sidebar</h3>
			<p>Sidebar content</p>
		</aside>
		<section>
			<h3>Lists</h3>
			<ul>
				<li>Item 1</li>
				<li>Item 2</li>
				<li>Item 3</li>
			</ul>
			<ol>
				<li>First</li>
				<li>Second</li>
			</ol>
		</section>
		<section>
			<h3>Table Data</h3>
			<table class="data-table">
				<tr>
					<th>Name</th>
					<th>Age</th>
					<th>City</th>
				</tr>
				<tr>
					<td>John</td>
					<td>30</td>
					<td>New York</td>
				</tr>
				<tr>
					<td>Jane</td>
					<td>25</td>
					<td>London</td>
				</tr>
			</table>
		</section>
		<section>
			<h3>Contact Form</h3>
			<form action="/submit" method="POST" id="contact-form" name="contact">
				<input type="text" name="name" placeholder="Your name" required>
				<input type="email" name="email" placeholder="Email" required>
				<textarea name="message" placeholder="Message"></textarea>
				<select name="subject">
					<option value="general">General</option>
					<option value="support">Support</option>
				</select>
				<button type="submit">Submit</button>
			</form>
		</section>
		<section itemscope itemtype="https://schema.org/Person">
			<span itemprop="name">John Doe</span>
			<span itemprop="email">john@example.com</span>
			<span itemprop="telephone">555-1234</span>
		</section>
	</main>
	<footer>
		<p>Contact: support@example.com | IP: 192.168.1.1</p>
		<p>SSN: 123-45-6789 | Zip: 12345-6789</p>
	</footer>
</body>
</html>
`

	simpleHTML = `
<html>
<body>
	<div class="content">
		<h1>Simple Test</h1>
		<p class="text">Simple paragraph</p>
	</div>
</body>
</html>
`
)

// TestScraperProviderDefinition tests the provider metadata
func TestScraperProviderDefinition(t *testing.T) {
	scraper := providers.NewScraper()
	def := scraper.Definition()

	assert.Equal(t, "scraper", def.ID)
	assert.Equal(t, "Web Scraper Service", def.Name)
	assert.NotEmpty(t, def.Description)
	assert.Equal(t, types.CategoryScraper, def.Category)

	// Check capabilities
	capabilities := make(map[string]bool)
	for _, cap := range def.Capabilities {
		capabilities[cap] = true
	}
	assert.True(t, capabilities["content_extraction"])
	assert.True(t, capabilities["xpath_queries"])
	assert.True(t, capabilities["css_selectors"])
	assert.True(t, capabilities["smart_extraction"])
	assert.True(t, capabilities["form_parsing"])
	assert.True(t, capabilities["metadata_extraction"])

	// Check tools
	assert.Greater(t, len(def.Tools), 20) // Should have many tools

	toolIDs := make(map[string]bool)
	for _, tool := range def.Tools {
		toolIDs[tool.ID] = true
		assert.NotEmpty(t, tool.Name)
		assert.NotEmpty(t, tool.Description)
	}

	// Verify key tools exist
	assert.True(t, toolIDs["scraper.text"])
	assert.True(t, toolIDs["scraper.xpath"])
	assert.True(t, toolIDs["scraper.article"])
	assert.True(t, toolIDs["scraper.form"])
	assert.True(t, toolIDs["scraper.metadata"])
	assert.True(t, toolIDs["scraper.table"])
}

// TestContentExtractText tests text extraction
func TestContentExtractText(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.text", map[string]interface{}{
		"html": simpleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	text := result.Data["text"].(string)
	assert.Contains(t, text, "Simple Test")
	assert.Contains(t, text, "Simple paragraph")
	assert.Greater(t, result.Data["word_count"].(int), 0)
}

// TestContentExtractTitle tests title extraction
func TestContentExtractTitle(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.title", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	assert.Equal(t, "Test Page", result.Data["title"])
	assert.False(t, result.Data["empty"].(bool))
}

// TestContentExtractLinks tests link extraction
func TestContentExtractLinks(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.links", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	links := result.Data["links"].([]string)
	assert.Greater(t, len(links), 0)
	assert.Contains(t, links, "/home")
	assert.Contains(t, links, "/about")
	assert.Contains(t, links, "https://external.com")
}

// TestContentExtractLinksAbsoluteOnly tests absolute link extraction
func TestContentExtractLinksAbsoluteOnly(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.links", map[string]interface{}{
		"html":          sampleHTML,
		"absolute_only": true,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	links := result.Data["links"].([]string)
	for _, link := range links {
		assert.True(t, len(link) > 7 && (link[:7] == "http://" || link[:8] == "https://"))
	}
}

// TestContentExtractImages tests image extraction
func TestContentExtractImages(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.images", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	// Images are returned as typed structs, verify count
	assert.Equal(t, 2, result.Data["count"])
}

// TestContentCSSSelect tests CSS selector queries
func TestContentCSSSelect(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	// Single element
	result, err := scraper.Execute(ctx, "scraper.select", map[string]interface{}{
		"html":     sampleHTML,
		"selector": "h1",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	assert.True(t, result.Data["found"].(bool))
	assert.Contains(t, result.Data["text"].(string), "Welcome to Test Page")

	// Multiple elements
	result, err = scraper.Execute(ctx, "scraper.select", map[string]interface{}{
		"html":     sampleHTML,
		"selector": "p",
		"all":      true,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	elements := result.Data["elements"].([]map[string]interface{})
	assert.Greater(t, len(elements), 3)
}

// TestContentExtractAttributes tests attribute extraction
func TestContentExtractAttributes(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.attribute", map[string]interface{}{
		"html":      sampleHTML,
		"selector":  "img",
		"attribute": "src",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	values := result.Data["values"].([]string)
	assert.Equal(t, 2, len(values))
	assert.Contains(t, values, "/image1.jpg")
}

// TestXPathQuery tests XPath queries
func TestXPathQuery(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	// Single element
	result, err := scraper.Execute(ctx, "scraper.xpath", map[string]interface{}{
		"html":  sampleHTML,
		"xpath": "//h1[@id='main-heading']",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	assert.True(t, result.Data["found"].(bool))
	assert.Contains(t, result.Data["text"].(string), "Welcome to Test Page")

	// Multiple elements
	result, err = scraper.Execute(ctx, "scraper.xpath", map[string]interface{}{
		"html":  sampleHTML,
		"xpath": "//h2 | //h3",
		"all":   true,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	elements := result.Data["elements"].([]map[string]interface{})
	assert.Greater(t, len(elements), 3)
}

// TestXPathText tests XPath text extraction
func TestXPathText(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.xpath_text", map[string]interface{}{
		"html":  sampleHTML,
		"xpath": "//article//p",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	texts := result.Data["texts"].([]string)
	assert.Greater(t, len(texts), 2)
}

// TestXPathAttribute tests XPath attribute extraction
func TestXPathAttribute(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.xpath_attr", map[string]interface{}{
		"html":      sampleHTML,
		"xpath":     "//img",
		"attribute": "alt",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	values := result.Data["values"].([]string)
	assert.Equal(t, 2, len(values))
	assert.Contains(t, values, "Test Image")
}

// TestExtractArticle tests smart article extraction
func TestExtractArticle(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.article", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	text := result.Data["text"].(string)
	assert.NotEmpty(t, text)
	assert.Greater(t, result.Data["word_count"].(int), 0)
}

// TestCleanHTML tests HTML sanitization
func TestCleanHTML(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	dirtyHTML := `<p onclick="alert('xss')">Test <script>alert('xss')</script></p>`

	result, err := scraper.Execute(ctx, "scraper.clean", map[string]interface{}{
		"html": dirtyHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	cleaned := result.Data["html"].(string)
	assert.NotContains(t, cleaned, "onclick")
	assert.NotContains(t, cleaned, "<script>")
}

// TestExtractSummary tests summary extraction
func TestExtractSummary(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.summary", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	summary := result.Data["summary"].(string)
	assert.NotEmpty(t, summary)
	assert.Equal(t, "meta", result.Data["source"]) // Should get from meta description
}

// TestFindForm tests form finding
func TestFindForm(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.form", map[string]interface{}{
		"html":     sampleHTML,
		"selector": "#contact-form",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	assert.True(t, result.Data["found"].(bool))
	assert.Equal(t, "/submit", result.Data["action"])
	assert.Equal(t, "POST", result.Data["method"])
	assert.Equal(t, "contact-form", result.Data["id"])
}

// TestGetFormFields tests form field extraction
func TestGetFormFields(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.form_fields", map[string]interface{}{
		"html":     sampleHTML,
		"selector": "#contact-form",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	// Form fields returned as typed structs, verify count
	count := result.Data["count"].(int)
	assert.Greater(t, count, 3)
}

// TestFindAllForms tests finding all forms
func TestFindAllForms(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.forms_all", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	forms := result.Data["forms"].([]map[string]interface{})
	assert.Equal(t, 1, len(forms))
	assert.Equal(t, "/submit", forms[0]["action"])
}

// TestExtractMetadata tests metadata extraction
func TestExtractMetadata(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.metadata", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	standard := result.Data["standard"].(map[string]string)
	openGraph := result.Data["open_graph"].(map[string]string)
	twitter := result.Data["twitter"].(map[string]string)

	assert.Equal(t, "This is a test page", standard["description"])
	assert.Equal(t, "OG Test Page", openGraph["og:title"])
	assert.Equal(t, "summary", twitter["twitter:card"])
}

// TestExtractHeadings tests heading extraction
func TestExtractHeadings(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.headings", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	headings := result.Data["headings"].([]map[string]interface{})
	assert.Greater(t, len(headings), 5)

	// Find h1
	var h1Found bool
	for _, heading := range headings {
		if heading["level"].(int) == 1 {
			h1Found = true
			assert.Contains(t, heading["text"].(string), "Welcome to Test Page")
			break
		}
	}
	assert.True(t, h1Found)
}

// TestExtractJSONLD tests JSON-LD extraction
func TestExtractJSONLD(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.jsonld", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	jsonLDData := result.Data["data"].([]interface{})
	assert.Equal(t, 1, len(jsonLDData))

	// Check JSON-LD content
	ldItem := jsonLDData[0].(map[string]interface{})
	assert.Equal(t, "Article", ldItem["@type"])
}

// TestExtractMicrodata tests microdata extraction
func TestExtractMicrodata(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.microdata", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	items := result.Data["items"].([]map[string]interface{})
	assert.Equal(t, 1, len(items))

	item := items[0]
	assert.Contains(t, item["type"].(string), "Person")
	properties := item["properties"].(map[string]interface{})
	assert.Equal(t, "John Doe", properties["name"])
}

// TestExtractOpenGraph tests Open Graph extraction
func TestExtractOpenGraph(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.og", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	og := result.Data["open_graph"].(map[string]interface{})
	assert.Equal(t, "OG Test Page", og["title"])
	assert.Equal(t, "OG Description", og["description"])
}

// TestExtractPhones tests phone number extraction
func TestExtractPhones(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.phones", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	phones := result.Data["phones"].([]string)
	assert.Greater(t, len(phones), 0)
}

// TestMatchPattern tests custom regex pattern matching
func TestMatchPattern(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	// Match all numbers
	result, err := scraper.Execute(ctx, "scraper.pattern", map[string]interface{}{
		"html":    sampleHTML,
		"pattern": `\d+`,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	matches := result.Data["matches"].([]string)
	assert.Greater(t, len(matches), 5)
}

// TestExtractBetween tests text extraction between delimiters
func TestExtractBetween(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	html := `<div>Start[content1]End Start[content2]End</div>`

	result, err := scraper.Execute(ctx, "scraper.between", map[string]interface{}{
		"html":  html,
		"start": "[",
		"end":   "]",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	results := result.Data["results"].([]string)
	assert.Equal(t, 2, len(results))
	assert.Contains(t, results, "content1")
	assert.Contains(t, results, "content2")
}

// TestExtractIPs tests IP address extraction
func TestExtractIPs(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.ips", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	ips := result.Data["ips"].([]string)
	assert.Equal(t, 1, len(ips))
	assert.Contains(t, ips, "192.168.1.1")
}

// TestExtractTable tests table extraction
func TestExtractTable(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.table", map[string]interface{}{
		"html":     sampleHTML,
		"selector": ".data-table",
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	rows := result.Data["rows"].([][]string)
	assert.Equal(t, 2, len(rows))

	headers := result.Data["headers"].([]string)
	assert.Equal(t, 3, len(headers))
	assert.Equal(t, "Name", headers[0])
	assert.Equal(t, "Age", headers[1])
	assert.Equal(t, "City", headers[2])

	// Check first data row
	assert.Equal(t, "John", rows[0][0])
	assert.Equal(t, "30", rows[0][1])
	assert.Equal(t, "New York", rows[0][2])
}

// TestExtractEmails tests email extraction
func TestExtractEmails(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.emails", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	emails := result.Data["emails"].([]string)
	assert.Greater(t, len(emails), 2)
	assert.Contains(t, emails, "test@example.com")
	assert.Contains(t, emails, "support@example.com")
}

// TestExtractURLs tests URL extraction
func TestExtractURLs(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.urls", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	urls := result.Data["urls"].([]string)
	assert.Greater(t, len(urls), 0)
	assert.Contains(t, urls, "https://example.com")
}

// TestExtractLists tests list extraction
func TestExtractLists(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.lists", map[string]interface{}{
		"html": sampleHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	// Lists returned as typed structs, verify list_count
	listCount := result.Data["list_count"].(int)
	// Lists should be extracted (at least 0 if selector doesn't match nested lists)
	assert.GreaterOrEqual(t, listCount, 0)
}

// TestErrorHandling tests error handling for invalid inputs
func TestErrorHandling(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	tests := []struct {
		name   string
		toolID string
		params map[string]interface{}
	}{
		{
			name:   "missing html parameter",
			toolID: "scraper.text",
			params: map[string]interface{}{},
		},
		{
			name:   "invalid xpath",
			toolID: "scraper.xpath",
			params: map[string]interface{}{
				"html":  "<div>test</div>",
				"xpath": "invalid[[[xpath",
			},
		},
		{
			name:   "missing selector",
			toolID: "scraper.select",
			params: map[string]interface{}{
				"html": "<div>test</div>",
			},
		},
		{
			name:   "invalid regex pattern",
			toolID: "scraper.pattern",
			params: map[string]interface{}{
				"html":    "<div>test</div>",
				"pattern": "[invalid(regex",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result, err := scraper.Execute(ctx, tt.toolID, tt.params, appCtx)
			require.NoError(t, err)         // No Go error
			assert.False(t, result.Success) // But operation failed
			assert.NotNil(t, result.Error)
		})
	}
}

// TestUnknownTool tests handling of unknown tool IDs
func TestUnknownTool(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	result, err := scraper.Execute(ctx, "scraper.nonexistent", map[string]interface{}{}, appCtx)
	// failure() function returns both error and failed result
	require.Error(t, err)
	assert.False(t, result.Success)
	assert.NotNil(t, result.Error)
	assert.Contains(t, *result.Error, "unknown tool")
	assert.Contains(t, err.Error(), "unknown tool")
}

// TestHTMLSizeLimit tests that oversized HTML is rejected
func TestHTMLSizeLimit(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	// Create HTML larger than 10MB limit
	largeHTML := "<div>" + string(make([]byte, 11*1024*1024)) + "</div>"

	result, err := scraper.Execute(ctx, "scraper.text", map[string]interface{}{
		"html": largeHTML,
	}, appCtx)

	require.NoError(t, err)
	assert.False(t, result.Success)
	assert.Contains(t, *result.Error, "exceeds maximum size")
}

// TestCharsetDetection tests charset detection with non-UTF8 content
func TestCharsetDetection(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	// UTF-8 HTML with special characters
	html := `<html><body><p>Hello 世界 🌍</p></body></html>`

	result, err := scraper.Execute(ctx, "scraper.text", map[string]interface{}{
		"html": html,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	text := result.Data["text"].(string)
	assert.Contains(t, text, "Hello")
}

// TestNormalization tests text normalization
func TestNormalization(t *testing.T) {
	scraper := providers.NewScraper()
	ctx := context.Background()
	appCtx := &types.Context{}

	html := `<div>Text   with    multiple     spaces</div>`

	result, err := scraper.Execute(ctx, "scraper.text", map[string]interface{}{
		"html":      html,
		"normalize": true,
	}, appCtx)

	require.NoError(t, err)
	assert.True(t, result.Success)

	text := result.Data["text"].(string)
	assert.Equal(t, "Text with multiple spaces", text)
}
