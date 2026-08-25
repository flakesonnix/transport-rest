package transportrest

import "strconv"

func strconvFormat(f float64) string { return strconv.FormatFloat(f, 'f', -1, 64) }
func intStr(n int) string            { return strconv.Itoa(n) }

func appendIfSet(params []queryParam, key string, value *string) []queryParam {
	if value != nil {
		params = append(params, queryParam{key, *value})
	}
	return params
}
