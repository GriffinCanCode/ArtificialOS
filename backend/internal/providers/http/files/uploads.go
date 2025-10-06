package files

import (
	"github.com/GriffinCanCode/AgentOS/backend/internal/providers/http/client"
	"context"
	"fmt"
	"os"

	"github.com/GriffinCanCode/AgentOS/backend/internal/shared/types"
)

// UploadsOps handles file upload operations
type UploadsOps struct {
	*client.HTTPOps
}

// GetTools returns upload tool definitions
func (u *UploadsOps) GetTools() []types.Tool {
	return []types.Tool{
		{
			ID:          "http.uploadFile",
			Name:        "Upload File",
			Description: "Upload file to URL via multipart form",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Upload URL", Required: true},
				{Name: "filepath", Type: "string", Description: "Local file path", Required: true},
				{Name: "fieldname", Type: "string", Description: "Form field name (default: file)", Required: false},
				{Name: "params", Type: "object", Description: "Additional form fields", Required: false},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
		{
			ID:          "http.uploadMultiple",
			Name:        "Upload Multiple Files",
			Description: "Upload multiple files in single request",
			Parameters: []types.Parameter{
				{Name: "url", Type: "string", Description: "Upload URL", Required: true},
				{Name: "files", Type: "array", Description: "Array of {path, fieldname} objects", Required: true},
				{Name: "params", Type: "object", Description: "Additional form fields", Required: false},
				{Name: "headers", Type: "object", Description: "HTTP headers", Required: false},
			},
			Returns: "object",
		},
	}
}

// UploadFile uploads single file using multipart form
func (u *UploadsOps) UploadFile(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	filePath, err := client.GetString(params, "filepath", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Check for context cancellation before starting
	select {
	case <-ctx.Done():
		return client.Failure(fmt.Sprintf("upload cancelled: %v", ctx.Err()))
	default:
	}

	// Verify file exists
	if _, err := os.Stat(filePath); err != nil {
		return client.Failure(fmt.Sprintf("file not found: %v", err))
	}

	fieldname, _ := client.GetString(params, "fieldname", false)
	if fieldname == "" {
		fieldname = "file"
	}

	// Create request with rate limiting
	req, err := u.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	// Add file to upload
	req.SetFile(fieldname, filePath)

	// Add additional form fields
	if formParams := client.GetMap(params, "params"); formParams != nil {
		for k, v := range formParams {
			req.SetFormData(map[string]string{
				k: fmt.Sprint(v),
			})
		}
	}

	// Add custom headers
	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	// Execute upload
	resp, err := req.Post(urlStr)
	if err != nil {
		return client.Failure(fmt.Sprintf("upload failed: %v", err))
	}

	result := client.ResponseToMap(resp)
	result["uploaded"] = true
	result["file"] = filePath

	return client.Success(result)
}

// UploadMultiple uploads multiple files in single request
func (u *UploadsOps) UploadMultiple(ctx context.Context, params map[string]interface{}, appCtx *types.Context) (*types.Result, error) {
	urlStr, err := client.GetString(params, "url", true)
	if err != nil {
		return client.Failure(err.Error())
	}

	filesParam := client.GetArray(params, "files")
	if len(filesParam) == 0 {
		return client.Failure("files array required and cannot be empty")
	}

	// Create request with rate limiting
	req, err := u.Client.Request(ctx)
	if err != nil {
		return client.Failure(err.Error())
	}

	uploadedFiles := []string{}

	// Add each file
	for i, fileParam := range filesParam {
		// Check for context cancellation in loop
		select {
		case <-ctx.Done():
			return client.Failure(fmt.Sprintf("upload cancelled: %v", ctx.Err()))
		default:
		}

		fileMap, ok := fileParam.(map[string]interface{})
		if !ok {
			return client.Failure(fmt.Sprintf("file[%d] must be object with {path, fieldname}", i))
		}

		filePath, err := client.GetString(fileMap, "path", true)
		if err != nil {
			return client.Failure(fmt.Sprintf("file[%d].path required", i))
		}

		// Verify file exists
		if _, err := os.Stat(filePath); err != nil {
			return client.Failure(fmt.Sprintf("file[%d] not found: %v", i, err))
		}

		fieldname, _ := client.GetString(fileMap, "fieldname", false)
		if fieldname == "" {
			fieldname = fmt.Sprintf("file%d", i)
		}

		// Add file using SetFiles for multiple files
		req.SetFile(fieldname, filePath)
		uploadedFiles = append(uploadedFiles, filePath)
	}

	// Add additional form fields
	if formParams := client.GetMap(params, "params"); formParams != nil {
		for k, v := range formParams {
			req.SetFormData(map[string]string{
				k: fmt.Sprint(v),
			})
		}
	}

	// Add custom headers
	if headers := client.GetMap(params, "headers"); headers != nil {
		for k, v := range headers {
			req.SetHeader(k, fmt.Sprint(v))
		}
	}

	// Execute upload
	resp, err := req.Post(urlStr)
	if err != nil {
		return client.Failure(fmt.Sprintf("upload failed: %v", err))
	}

	result := client.ResponseToMap(resp)
	result["uploaded"] = len(uploadedFiles)
	result["files"] = uploadedFiles

	return client.Success(result)
}
