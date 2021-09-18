package int

import (
	"fmt"
	"strings"

	"github.com/onsi/gomega/format"
	"github.com/onsi/gomega/types"
)

func toError(x interface{}) (error, bool) {
	err, ok := x.(error)
	if (!ok) {
		return nil, false
	}

	return err, true
}

type MatchBackendErrorMatcher struct {
	Error error
}

func (matcher *MatchBackendErrorMatcher) Match(actual interface{}) (success bool, err error) {
	err, ok := toError(actual)

	if (!ok) {
		return false, fmt.Errorf("MatchBackendError matcher required an error, Got:\n%s", format.Object(actual, 1))
	}

	return strings.Contains(err.Error(), matcher.Error.Error()), nil
}

func (matcher *MatchBackendErrorMatcher) FailureMessage(actual interface{}) (message string) {
	return format.Message(actual, "to be", matcher.Error.Error())
}

func (matcher *MatchBackendErrorMatcher) NegatedFailureMessage(actual interface{}) (message string) {
	return format.Message(actual, "not to be", matcher.Error.Error())
}

func MatchBackendError(error error) types.GomegaMatcher {
	return &MatchBackendErrorMatcher{
		Error: error,
	}
}
