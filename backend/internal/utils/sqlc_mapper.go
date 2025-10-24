package utils

import (
	"encoding/json"
)

func MapAdditionalInfo(src []byte) (map[string]any, error) {
	if len(src) == 0 {
		return nil, nil
	}

	additionalInfo := make(map[string]any, 0)
	err := json.Unmarshal(src, &additionalInfo)
	if err != nil {
		return nil, err
	}
	return additionalInfo, nil
}

func MapAdditionalInfoToByte(src map[string]any) ([]byte, error) {
	if src == nil {
		return nil, nil
	}

	if len(src) == 0 {
		return nil, nil
	}

	additionalInfo, err := json.Marshal(src)
	if err != nil {
		return nil, err
	}

	return additionalInfo, nil
}
