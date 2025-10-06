package http

import (
	"encoding/json"
)

// parseJSON parses JSON string into interface
func parseJSON(jsonStr string, v interface{}) error {
	return json.Unmarshal([]byte(jsonStr), v)
}

// toJSON converts interface to JSON string
func toJSON(v interface{}) (string, error) {
	bytes, err := json.Marshal(v)
	if err != nil {
		return "", err
	}
	return string(bytes), nil
}
